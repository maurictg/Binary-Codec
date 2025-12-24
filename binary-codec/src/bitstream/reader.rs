use std::cmp::min;

use crate::{DeserializationError, encoding::fixed_int::FixedInt};

pub struct BitStreamReader<'a> {
    buffer: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitStreamReader<'a> {
    /// Create a new LSB-first reader
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, bit_pos: 0 }
    }

    /// Get byte position of reader
    pub fn byte_pos(&self) -> usize {
        self.bit_pos / 8
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
            if self.byte_pos() >= self.buffer.len() {
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
            let byte_val = self.buffer[self.byte_pos()];

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
        }

        Ok(result)
    }

    /// Read a full byte, aligning to the next byte boundary
    pub fn read_byte(&mut self) -> Result<u8, DeserializationError> {
        self.align_byte();

        if self.byte_pos() >= self.buffer.len() {
            return Err(DeserializationError::NotEnoughBytes(1));
        }

        let b = self.buffer[self.byte_pos()];
        self.bit_pos += 8;
        Ok(b)
    }

    /// Read a slice of bytes, aligning first
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], DeserializationError> {
        self.align_byte();

        let start = self.byte_pos();
        if start + count > self.buffer.len() {
            return Err(DeserializationError::NotEnoughBytes(start + count - self.buffer.len()));
        }

        self.bit_pos += 8 * count;

        Ok(&self.buffer[start..start + count])
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
    pub fn read_fixed_int<const S : usize, T : FixedInt<S>>(&mut self) -> Result<T, DeserializationError> {
        let data = self.read_bytes(S)?;
        Ok(FixedInt::deserialize(data))
    }

    /// Align the reader to the next byte boundary
    pub fn align_byte(&mut self) {
        let rem = self.bit_pos % 8;
        if rem != 0 {
            self.bit_pos += 8 - rem;
        }
    }

    /// Reset reading position
    pub fn reset(&mut self) {
        self.bit_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::DeserializationError;

    use super::BitStreamReader;

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
        assert_eq!(reader.read_bit(), Err(DeserializationError::NotEnoughBytes(1)));
        assert_eq!(reader.read_byte(), Err(DeserializationError::NotEnoughBytes(1)));
        assert_eq!(reader.read_bytes(2), Err(DeserializationError::NotEnoughBytes(2)));
    }

    #[test]
    fn test_multiple_operations() {
        let buf = [0b10101010, 0b11001100, 0xFF, 0x00];
        let mut reader = BitStreamReader::new(&buf);

        assert_eq!(reader.read_bit(), Ok(false)); // bit 0
        assert_eq!(reader.read_small(3), Ok(0b101)); // bits 1-3
        assert_eq!(reader.read_byte(), Ok(0b11001100)); // aligned full byte
        assert_eq!(reader.read_bytes(2), Ok(&[0xFF, 0x00][..]));
        assert_eq!(reader.read_bit(), Err(DeserializationError::NotEnoughBytes(1)));
    }

    #[test]
    fn test_read_dyn_int() {
        let buf = vec![0, 127, 128, 1, 255, 255, 255, 127];
        let mut stream = BitStreamReader::new(&buf);

        assert_eq!(Ok(0), stream.read_byte());
        assert_eq!(Ok(127), stream.read_dyn_int());
        assert_eq!(Ok(128), stream.read_dyn_int());
        assert_eq!(Ok(268435455), stream.read_dyn_int());
        assert_eq!(Err(DeserializationError::NotEnoughBytes(1)), stream.read_dyn_int());
    }
}
