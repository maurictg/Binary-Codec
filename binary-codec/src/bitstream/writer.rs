use std::cmp::min;

use crate::{CryptoStream, encoding::fixed_int::FixedInt};

pub struct BitStreamWriter<'a> {
    buffer: &'a mut Vec<u8>,
    bit_pos: usize,
    crypto: Option<Box<dyn CryptoStream>>,
}

impl<'a> BitStreamWriter<'a> {
    /// Create a new LSB-first writer
    pub fn new(buffer: &'a mut Vec<u8>) -> Self {
        Self {
            buffer,
            bit_pos: 0,
            crypto: None,
        }
    }

    /// Return slice of buffer
    pub fn slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Set crypto stream
    pub fn set_crypto(&mut self, crypto: Option<Box<dyn CryptoStream>>) {
        self.crypto = crypto;
    }

    /// Get byte position of writer
    pub fn byte_pos(&self) -> usize {
        self.bit_pos / 8
    }

    /// Write a single bit
    pub fn write_bit(&mut self, val: bool) {
        self.write_small(val as u8, 1);
    }

    /// Write 1-8 bits (LSB-first)
    pub fn write_small(&mut self, mut val: u8, mut bits: u8) {
        assert!(bits > 0 && bits < 8);

        while bits > 0 {
            self.ensure_byte();

            // Determine the current bit position inside the current byte (0..7)
            let bit_offset = self.bit_pos % 8;

            // Determine how many bytes left to read. Min(bits left in this byte, bits to write)
            let bits_in_current_byte = min(8 - bit_offset as u8, bits);

            // Create a mask for the bits we are going to modify.
            // Example: bit_offset = 2, bits_in_current_byte = 3
            // mask = 00011100 (we will only touch bits 2,3,4)
            let mask = ((1 << bits_in_current_byte) - 1) << bit_offset;

            // Prepare the bits to write:
            // 1️⃣ Take only the lowest 'bits_in_current_byte' bits from val
            // 2️⃣ Shift them left by bit_offset to align inside the byte
            // Example: val = 0b110101, bits_in_current_byte = 3, bit_offset = 2
            // Step 1: val & 0b111 = 0b101
            // Step 2: 0b101 << 2 = 0b10100 → shifted_val
            let shifted_val = (val & ((1 << bits_in_current_byte) - 1)) << bit_offset;

            let byte_pos = self.byte_pos();

            // Clear the bits in this byte where we will write
            self.buffer[byte_pos] &= !mask;

            // Write the new bits into the cleared positions
            self.buffer[byte_pos] |= shifted_val & mask;

            // Decrease the number of bits remaining to write
            bits -= bits_in_current_byte;

            // Shift val right to remove the bits we've already written
            val >>= bits_in_current_byte;

            self.bit_pos += bits_in_current_byte as usize;

            // If full byte, encrypt it (if needed)
            if self.bit_pos % 8 == 0 {
                if let Some(crypto) = self.crypto.as_mut() {
                    let b = self.buffer[byte_pos];
                    self.buffer[byte_pos] = crypto.apply_keystream_byte(b);
                }
            }
        }
    }

    /// Write a full byte, starting at the next byte boundary
    pub fn write_byte(&mut self, byte: u8) {
        self.align_byte();
        self.ensure_byte();

        let byte_pos = self.byte_pos();
        let byte = if let Some(crypto) = self.crypto.as_mut() {
            crypto.apply_keystream_byte(byte)
        } else {
            byte
        };

        self.buffer[byte_pos] = byte;
        self.bit_pos += 8;
    }

    /// Write a slice of bytes, starting at the next byte boundary
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.align_byte();

        if let Some(crypto) = self.crypto.as_mut() {
            let encrypted = crypto.apply_keystream(data);
            self.buffer.extend_from_slice(encrypted);
        } else {
            self.buffer.extend_from_slice(data);
        }

        self.bit_pos += 8 * data.len();
    }

    /// Write a dynamic int, starting at the next byte bounary
    /// The last bit is used as a continuation flag for the next byte
    pub fn write_dyn_int(&mut self, mut val: u128) {
        while val > 0 {
            let mut encoded = val % 128;
            val /= 128;
            if val > 0 {
                encoded |= 128;
            }
            self.write_byte(encoded as u8);
        }
    }

    /// Write a integer of fixed size to the buffer
    pub fn write_fixed_int<const S: usize, T: FixedInt<S>>(&mut self, val: T) {
        self.write_bytes(&val.serialize());
    }

    /// Ensure the buffer has at least `byte_pos + 1` bytes
    fn ensure_byte(&mut self) {
        let byte_pos = self.byte_pos();
        if byte_pos >= self.buffer.len() {
            self.buffer.resize(byte_pos + 1, 0);
        }
    }

    /// Align the stream to the next byte boundary
    pub fn align_byte(&mut self) {
        let rem = self.bit_pos % 8;
        if rem != 0 {
            let byte_pos = self.byte_pos();
            self.bit_pos += 8 - rem;

            // Encrypt byte
            if let Some(crypto) = self.crypto.as_mut() {
                self.buffer[byte_pos] = crypto.apply_keystream_byte(self.buffer[byte_pos]);
            }
        }
    }

    /// Reset writing position
    pub fn reset(&mut self) {
        self.bit_pos = 0;
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::CryptoStream;

    use super::BitStreamWriter;

    struct PlusOneEncrypter {
        ciphertext: Vec<u8>
    }

    impl CryptoStream for PlusOneEncrypter {
        fn apply_keystream_byte(&mut self, b: u8) -> u8 {
            self.ciphertext.push(b + 1);
            *self.ciphertext.last().unwrap()
        }
    
        fn apply_keystream(&mut self, slice: &[u8]) -> &[u8] {
            let d = slice.iter().map(|s|s + 1);
            self.ciphertext.extend(d);
            &self.ciphertext[self.ciphertext.len() - slice.len()..]
        }
    }

    #[test]
    fn test_encrypt_bytes() {
        let mut buf = Vec::new();
        let mut writer = BitStreamWriter::new(&mut buf);
        writer.crypto = Some(Box::new(PlusOneEncrypter { ciphertext: Vec::new() }));

        writer.write_byte(1);
        writer.write_byte(2);
        writer.write_byte(3);
        writer.write_bit(false);
        writer.write_bit(false);
        writer.write_bit(true);
        writer.write_bytes(&[5,6,7,8,9]);
        writer.write_byte(10);

        assert_eq!(buf, vec![2,3,4,5,6,7,8,9,10,11]);
    }


    /// Helper to format buffer as binary strings
    fn buffer_to_bin(buffer: &[u8]) -> Vec<String> {
        buffer.iter().map(|b| format!("{:08b}", b)).collect()
    }

    #[test]
    fn test_write_bit() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_bit(true);
        stream.write_bit(false);
        stream.write_bit(true);
        stream.write_bit(true); // 4 bits

        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 0b00001101); // LSB-first: first bit is lowest
    }

    #[test]
    fn test_write_small() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_small(0b101, 3); // write 3 bits
        stream.write_small(0b11, 2); // write 2 bits
        stream.write_small(0b111, 3); // write 3 bits

        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 0b11111101); // bits packed LSB-first
    }

    #[test]
    fn test_write_cross_byte() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        // write 11 bits: first 7 bits fill almost one byte, next 3 bits spill into second byte
        stream.write_small(0b00101011, 7);
        stream.write_small(0b1101, 4);

        assert_eq!(buf.len(), 2);
        assert_eq!(buf[0], 0b10101011);
        assert_eq!(buf[1], 0b00000110);
    }

    #[test]
    fn test_write_byte() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_bit(true); // 1 bit
        stream.write_byte(0xAA); // should align and write full byte

        assert_eq!(buf.len(), 2);
        assert_eq!(buf[0], 0b00000001); // first bit written, rest padded
        assert_eq!(buf[1], 0xAA); // full byte
    }

    #[test]
    fn test_write_bytes() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_bit(true); // 1 bit
        stream.write_bytes(&[0xAA, 0xBB, 0xCC]); // align and write slice

        assert_eq!(buf.len(), 4);
        assert_eq!(buf[0], 0b00000001); // padding of first bit
        assert_eq!(buf[1], 0xAA);
        assert_eq!(buf[2], 0xBB);
        assert_eq!(buf[3], 0xCC);
    }

    #[test]
    fn test_alignment() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_small(0b11, 2); // 2 bits
        stream.align_byte();
        stream.write_byte(0xFF);

        assert_eq!(buf.len(), 2);
        assert_eq!(buf[0], 0b00000011); // 2 bits written, rest padded
        assert_eq!(buf[1], 0xFF);
    }

    #[test]
    fn test_multiple_operations() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_bit(true);
        stream.write_small(0b101, 3);
        stream.write_byte(0xAA);
        stream.write_bytes(&[0xBB, 0xCC]);
        stream.write_small(0b11, 2);

        let bin = buffer_to_bin(&buf);
        println!("{:?}", bin);

        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0b00001011); // first 4 bits
        assert_eq!(buf[1], 0xAA); // write_byte
        assert_eq!(buf[2], 0xBB);
        assert_eq!(buf[3], 0xCC);
        assert_eq!(buf[4], 0b00000011); // last 2 bits
    }

    #[test]
    fn test_write_dyn_int() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_dyn_int(127);
        assert_eq!(1, stream.len());

        stream.write_dyn_int(128); // Crossed 127 = boundary of first byte
        assert_eq!(3, stream.len());

        stream.write_dyn_int(268435455); // 4 bytes boundary
        assert_eq!(7, stream.len());

        assert_eq!(vec![127, 128, 1, 255, 255, 255, 127], buf);
    }

    #[test]
    fn test_write_fixed_int() {
        let mut buf = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buf);

        stream.write_fixed_int(1u8);
        stream.write_fixed_int(1i8);
        stream.write_fixed_int(2u16);
        stream.write_fixed_int(2i16);
        stream.write_fixed_int(3u32);
        stream.write_fixed_int(3i32);
        stream.write_fixed_int(4u64);
        stream.write_fixed_int(4i64);
        stream.write_fixed_int(5u128);
        stream.write_fixed_int(5i128);

        assert_eq!(
            vec![
                1, 2, 0, 2, 0, 4, 0, 0, 0, 3, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0,
                0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 10
            ],
            buf
        );
    }
}
