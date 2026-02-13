use std::cmp::min;

use crate::{DeserializationError, bitstream::CryptoStream, encoding::fixed_int::FixedInt};

pub struct BitStreamReader<'a> {
    buffer: &'a [u8],
    bit_pos: usize,
    last_read_byte: Option<u8>,
    offset_end: usize,
    crypto: Option<Box<dyn CryptoStream>>,
    marker: Option<usize>,
}

impl<'a> BitStreamReader<'a> {
    /// Create a new LSB-first reader
    ///
    /// # Properties
    /// - `buffer`: The buffer to read from
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            bit_pos: 0,
            crypto: None,
            offset_end: 0,
            last_read_byte: None,
            marker: None,
        }
    }

    /// Return slice from reading position (or start, if `from_start` is true) to offset-end
    pub fn slice(&self, from_start: bool) -> &[u8] {
        let start = if from_start { 0 } else { self.byte_pos() };

        &self.buffer[start..self.buffer.len() - self.offset_end]
    }

    /// Set marker at current byte
    pub fn set_marker(&mut self) {
        self.marker = Some(self.byte_pos());
    }

    /// Unset marker
    pub fn reset_marker(&mut self) {
        self.marker = None;
    }

    /// Return slice from marker (or start if marker is unset) to specific position or current byte
    /// If crypto is set, it will return the decrypted slice. This is different from other `slice` methods which always returns the raw buffer slice.
    pub fn slice_marker(&self, to: Option<usize>) -> &[u8] {
        let start = self.marker.unwrap_or(0);
        let end = to.unwrap_or(self.byte_pos());

        if let Some(crypto) = self.crypto.as_ref() {
            return &crypto.get_plaintext()[start..end];
        }

        &self.buffer[start..end]
    }

    /// Return slice from offset-end to end of buffer
    /// If crypto is set, it will apply the keystream to the slice before returning.
    /// This is different from other `slice` methods which always returns the raw buffer slice.
    pub fn slice_end(&mut self) -> &[u8] {
        let slice = &self.buffer[self.buffer.len() - self.offset_end..];

        if let Some(crypto) = self.crypto.as_mut() {
            crypto.apply_keystream(slice)
        } else {
            slice
        }
    }

    /// Set crypto stream
    pub fn set_crypto(&mut self, crypto: Option<Box<dyn CryptoStream>>) {
        self.crypto = crypto;
    }

    /// Set integrity offset to ignore when reading
    pub fn set_offset_end(&mut self, len: usize) {
        self.offset_end = len;
    }

    /// Get byte position of reader
    pub fn byte_pos(&self) -> usize {
        self.bit_pos / 8
    }

    /// Get current byte, from last_read_byte cache or from buffer
    fn current_byte(&mut self) -> u8 {
        if let Some(b) = self.last_read_byte {
            b
        } else {
            let mut b = self.buffer[self.byte_pos()];
            if let Some(crypto) = self.crypto.as_mut() {
                b = crypto.apply_keystream_byte(b);
            }

            self.last_read_byte = Some(b);
            b
        }
    }

    /// Read a single bit
    pub fn read_bit(&mut self) -> Result<bool, DeserializationError> {
        self.read_small(1).map(|v| v != 0)
    }

    /// Read 1-8 bits as u8 (LSB-first)
    pub fn read_small(&mut self, mut bits: u8) -> Result<u8, DeserializationError> {
        assert!(bits > 0 && bits < 8);

        let mut result: u8 = 0;
        let mut shift = 0;

        while bits > 0 {
            if self.byte_pos() >= self.buffer.len() - self.offset_end {
                return Err(DeserializationError::NotEnoughBytes(1));
            }

            // Find the bit position inside the current byte (0..7)
            let bit_offset = self.bit_pos % 8;

            // Determine how many bytes left to read. Min(bits left in this byte, bits to read)
            let bits_in_current_byte = min(8 - bit_offset as u8, bits);

            // Create a mask to isolate the bits we want from this byte.
            // Example: if bit_offset = 2 and bits_in_current_byte = 3,
            // mask = 00011100 (only bits 2,3,4 are 1)
            let mask = ((1 << bits_in_current_byte) - 1) << bit_offset;
            let byte_val = self.current_byte();

            // Apply the mask to isolate the bits and shift them to LSB
            // Example: byte_val = 10101100, mask = 00011100
            // (10101100 & 00011100) >> 2 = 00000101
            let val = (byte_val & mask) >> bit_offset;

            // Merge the extracted bits into the final result.
            // Shift them to the correct position based on how many bits we already read.
            result |= val << shift;

            // Decrease the remaining bits we need to read
            bits -= bits_in_current_byte;

            // Update the shift for the next batch of bits (if crossing byte boundary)
            shift += bits_in_current_byte;

            self.bit_pos += bits_in_current_byte as usize;

            // If crossed byte boundary, reset last read byte
            if self.bit_pos % 8 == 0 {
                self.last_read_byte = None;
            }
        }

        Ok(result)
    }

    /// Read a full byte, aligning to the next byte boundary
    pub fn read_byte(&mut self) -> Result<u8, DeserializationError> {
        self.align_byte();

        if self.byte_pos() >= self.buffer.len() - self.offset_end {
            return Err(DeserializationError::NotEnoughBytes(1));
        }

        let byte = self.current_byte();
        self.bit_pos += 8;
        self.last_read_byte = None;

        Ok(byte)
    }

    /// Read a slice of bytes, aligning first
    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], DeserializationError> {
        self.align_byte();

        let start = self.byte_pos();
        if start + count > self.buffer.len() - self.offset_end {
            return Err(DeserializationError::NotEnoughBytes(
                start + count - self.buffer.len(),
            ));
        }

        self.bit_pos += 8 * count;
        self.last_read_byte = None;

        let slice = &self.buffer[start..start + count];
        if let Some(crypto) = self.crypto.as_mut() {
            Ok(crypto.apply_keystream(slice))
        } else {
            Ok(slice)
        }
    }

    /// Read a dynamic int, starting at the next byte bounary
    /// The last bit is used as a continuation flag for the next byte
    pub fn read_dyn_int(&mut self) -> Result<u128, DeserializationError> {
        self.align_byte();
        let mut num: u128 = 0;
        let mut multiplier: u128 = 1;

        loop {
            let byte = self.read_byte()?; // None if EOF
            num += ((byte & 127) as u128) * multiplier;

            // If no continuation bit, stop
            if (byte & 1 << 7) == 0 {
                break;
            }

            multiplier *= 128;
        }

        Ok(num)
    }

    /// Read a integer of fixed size from the buffer
    pub fn read_fixed_int<const S: usize, T: FixedInt<S>>(
        &mut self,
    ) -> Result<T, DeserializationError> {
        let data = self.read_bytes(S)?;
        Ok(FixedInt::deserialize(data))
    }

    /// Align the reader to the next byte boundary
    pub fn align_byte(&mut self) {
        let rem = self.bit_pos % 8;
        if rem != 0 {
            self.bit_pos += 8 - rem;
            self.last_read_byte = None;
        }
    }

    /// Get bytes left
    pub fn bytes_left(&self) -> usize {
        let left = self.buffer.len() - self.byte_pos() - self.offset_end;
        if self.bit_pos % 8 != 0 {
            left - 1 // If not aligned, we can't read the last byte fully
        } else {
            left
        }
    }

    /// Reset reading position
    pub fn reset(&mut self) {
        self.bit_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::{DeserializationError, bitstream::CryptoStream};

    use super::BitStreamReader;

    struct PlusOneDecrypter {
        plain: Vec<u8>,
    }

    impl CryptoStream for PlusOneDecrypter {
        fn apply_keystream_byte(&mut self, b: u8) -> u8 {
            self.plain.push(b + 1);
            *self.plain.last().unwrap()
        }

        fn apply_keystream(&mut self, slice: &[u8]) -> &[u8] {
            let d = slice.iter().map(|s| s + 1);
            self.plain.extend(d);
            &self.plain[self.plain.len() - slice.len()..]
        }

        fn get_plaintext(&self) -> &[u8] {
            &self.plain
        }
    }

    #[test]
    fn test_decrypt_bytes() {
        let buf = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut reader = BitStreamReader::new(&buf);
        reader.crypto = Some(Box::new(PlusOneDecrypter { plain: Vec::new() }));

        assert_eq!(reader.read_byte(), Ok(2));
        assert_eq!(reader.read_byte(), Ok(3));
        assert_eq!(reader.read_byte(), Ok(4));
        // 4 = 00000100, +1 = 00000101
        assert_eq!(reader.read_bit(), Ok(true));
        assert_eq!(reader.read_bit(), Ok(false));
        assert_eq!(reader.read_bit(), Ok(true));
        assert_eq!(reader.read_bytes(5), Ok(&[6, 7, 8, 9, 10][..]));
        assert_eq!(reader.read_byte(), Ok(11));
    }

    /// Helper to build buffers
    fn make_buffer() -> Vec<u8> {
        vec![0b10101100, 0b11010010, 0xFF, 0x00]
    }

    #[test]
    fn test_read_single_bits() {
        let buf = make_buffer();
        let mut reader = BitStreamReader::new(&buf);

        // LSB-first: read bits starting from least significant
        assert_eq!(reader.read_bit(), Ok(false));
        assert_eq!(reader.read_bit(), Ok(false));
        assert_eq!(reader.read_bit(), Ok(true));
        assert_eq!(reader.read_bit(), Ok(true));
        assert_eq!(reader.read_bit(), Ok(false));
        assert_eq!(reader.read_bit(), Ok(true));
        assert_eq!(reader.read_bit(), Ok(false));
        assert_eq!(reader.read_bit(), Ok(true));
    }

    #[test]
    fn test_read_small() {
        let buf = [0b10101100, 0b11010010];
        let mut reader = BitStreamReader::new(&buf);

        assert_eq!(reader.read_small(3), Ok(0b100));
        assert_eq!(reader.read_small(4), Ok(0b0101));
        assert_eq!(reader.read_small(1), Ok(0b1));
        assert_eq!(reader.read_small(4), Ok(0b0010));
    }

    #[test]
    fn test_read_cross_byte() {
        let buf = [0b10101100, 0b11010001];
        let mut reader = BitStreamReader::new(&buf);

        // Read first 10 bits (crosses into second byte)
        assert_eq!(reader.read_small(7), Ok(0b00101100));
        assert_eq!(reader.read_small(3), Ok(0b011));
    }

    #[test]
    fn test_read_byte() {
        let buf = [0b10101100, 0b11010010];
        let mut reader = BitStreamReader::new(&buf);

        reader.read_small(3).unwrap(); // advance 3 bits
        assert_eq!(reader.read_byte(), Ok(0b11010010)); // full second byte
    }

    #[test]
    fn test_read_bytes() {
        let buf = [0x01, 0xAA, 0xBB, 0xCC];
        let mut reader = BitStreamReader::new(&buf);

        reader.read_bit().unwrap(); // first bit
        let slice = reader.read_bytes(3).unwrap();
        assert_eq!(slice, &[0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_align_byte() {
        let buf = [0b10101100, 0b11010010];
        let mut reader = BitStreamReader::new(&buf);

        reader.read_small(3).unwrap(); // 3 bits
        reader.align_byte(); // move to next byte
        assert_eq!(reader.read_byte(), Ok(0b11010010));
    }

    #[test]
    fn test_eof_behavior() {
        let buf = [0xFF];
        let mut reader = BitStreamReader::new(&buf);

        assert_eq!(reader.read_byte(), Ok(0xFF));
        assert_eq!(
            reader.read_bit(),
            Err(DeserializationError::NotEnoughBytes(1))
        );
        assert_eq!(
            reader.read_byte(),
            Err(DeserializationError::NotEnoughBytes(1))
        );
        assert_eq!(
            reader.read_bytes(2),
            Err(DeserializationError::NotEnoughBytes(2))
        );
    }

    #[test]
    fn test_multiple_operations() {
        let buf = [0b10101010, 0b11001100, 0xFF, 0x00];
        let mut reader = BitStreamReader::new(&buf);

        assert_eq!(reader.read_bit(), Ok(false)); // bit 0
        assert_eq!(reader.read_small(3), Ok(0b101)); // bits 1-3
        assert_eq!(reader.read_byte(), Ok(0b11001100)); // aligned full byte
        assert_eq!(reader.read_bytes(2), Ok(&[0xFF, 0x00][..]));
        assert_eq!(
            reader.read_bit(),
            Err(DeserializationError::NotEnoughBytes(1))
        );
    }

    #[test]
    fn test_read_dyn_int() {
        let buf = vec![0, 127, 128, 1, 255, 255, 255, 127];
        let mut stream = BitStreamReader::new(&buf);

        assert_eq!(Ok(0), stream.read_byte());
        assert_eq!(Ok(127), stream.read_dyn_int());
        assert_eq!(Ok(128), stream.read_dyn_int());
        assert_eq!(Ok(268435455), stream.read_dyn_int());
        assert_eq!(
            Err(DeserializationError::NotEnoughBytes(1)),
            stream.read_dyn_int()
        );
    }

    #[test]
    fn test_read_fixed_int() {
        let buf = vec![
            1, 2, 0, 2, 0, 4, 0, 0, 0, 3, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
            8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 10,
        ];

        let mut stream = BitStreamReader::new(&buf);
        let v1: u8 = stream.read_fixed_int().unwrap();
        let v2: i8 = stream.read_fixed_int().unwrap();
        let v3: u16 = stream.read_fixed_int().unwrap();
        let v4: i16 = stream.read_fixed_int().unwrap();
        let v5: u32 = stream.read_fixed_int().unwrap();
        let v6: i32 = stream.read_fixed_int().unwrap();
        let v7: u64 = stream.read_fixed_int().unwrap();
        let v8: i64 = stream.read_fixed_int().unwrap();
        let v9: u128 = stream.read_fixed_int().unwrap();
        let v10: i128 = stream.read_fixed_int().unwrap();

        assert_eq!(v1, 1);
        assert_eq!(v2, 1);
        assert_eq!(v3, 2);
        assert_eq!(v4, 2);
        assert_eq!(v5, 3);
        assert_eq!(v6, 3);
        assert_eq!(v7, 4);
        assert_eq!(v8, 4);
        assert_eq!(v9, 5);
        assert_eq!(v10, 5);
    }

    #[test]
    fn test_bytes_left() {
        let buf = [0b10101100, 0b11010010, 0xFF, 0x00];
        let mut reader = BitStreamReader::new(&buf);

        assert_eq!(reader.bytes_left(), 4);
        reader.read_small(3).unwrap(); // read 3 bits
        assert_eq!(reader.bytes_left(), 3); // 3 full bytes left
        reader.read_byte().unwrap(); // read one byte
        assert_eq!(reader.bytes_left(), 2); // now 2 bytes left
        reader.read_byte().unwrap(); // read another byte
        assert_eq!(reader.bytes_left(), 1); // now 1 bytes left
        reader.read_bit().unwrap(); // read one bit
        assert_eq!(reader.bytes_left(), 0); // no full bytes left
    }

    #[test]
    fn offset_end_ignores_bytes_and_can_slice() {
        let buff = [1, 2, 3, 4, 5];
        let mut reader = BitStreamReader::new(&buff);

        reader.set_offset_end(2);
        assert_eq!(reader.bytes_left(), 3);
        assert_eq!(reader.read_byte(), Ok(1));

        assert_eq!(reader.slice(true), &[1, 2, 3]);
        assert_eq!(reader.slice(false), &[2, 3]);
        assert_eq!(reader.slice_end(), &[4, 5]);

        assert_eq!(reader.read_byte(), Ok(2));
        assert_eq!(reader.read_byte(), Ok(3));
        assert_eq!(
            reader.read_byte(),
            Err(DeserializationError::NotEnoughBytes(1))
        );

        reader.set_offset_end(0);
        assert_eq!(reader.bytes_left(), 2);
        assert_eq!(reader.read_byte(), Ok(4));
        assert_eq!(reader.read_byte(), Ok(5));
    }

    #[test]
    fn test_slice_start() {
        let buff = [10, 20, 30, 40, 50];
        let mut reader = BitStreamReader::new(&buff);

        assert_eq!(reader.slice_marker(None), &[]);

        reader.read_byte().unwrap(); // Read 10
        assert_eq!(reader.slice_marker(None), &[10]);

        reader.read_small(4).unwrap(); // Read 4 bits of 20
        assert_eq!(reader.slice_marker(None), &[10]);

        reader.read_small(4).unwrap(); // Read remaining 4 bits of 20
        assert_eq!(reader.slice_marker(None), &[10, 20]);

        reader.read_bytes(2).unwrap(); // Read 30, 40
        assert_eq!(reader.slice_marker(None), &[10, 20, 30, 40]);
    }

    #[test]
    fn test_slice_start_with_marker() {
        let buff = [10, 20, 30, 40, 50];
        let mut reader = BitStreamReader::new(&buff);

        reader.read_byte().unwrap(); // Read 10
        assert_eq!(reader.slice_marker(None), &[10]);
        reader.set_marker();
        assert_eq!(reader.slice_marker(None), &[]);

        reader.read_bytes(2).unwrap(); // Read 20, 30
        assert_eq!(reader.slice_marker(None), &[20, 30]);
    }
}
