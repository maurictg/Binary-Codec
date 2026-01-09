pub mod reader;
pub mod writer;

pub trait CryptoStream {
    fn decrypt_byte(&mut self, b: u8) -> u8;
    fn decrypt_slice(&mut self, slice: &[u8]) -> &[u8];
}

#[cfg(test)]
mod tests {
    use crate::bitstream::{reader::BitStreamReader, writer::BitStreamWriter};

    #[test]
    fn can_serialize_and_deserialize_using_bitstream() {
        let mut buffer = Vec::new();
        let mut writer = BitStreamWriter::new(&mut buffer);

        writer.write_bit(true);
        writer.write_bit(false);
        writer.write_bit(true);
        writer.write_byte(12);
        writer.write_bytes(&[34, 56]);

        assert_eq!(writer.byte_pos(), 4);

        let mut reader = BitStreamReader::new(&buffer);
        assert_eq!(reader.read_bit().unwrap(), true);
        assert_eq!(reader.read_bit().unwrap(), false);
        assert_eq!(reader.read_bit().unwrap(), true);
        assert_eq!(reader.read_byte().unwrap(), 12);
        assert_eq!(reader.read_byte().unwrap(), 34);
        assert_eq!(reader.read_byte().unwrap(), 56);

        assert_eq!(buffer, [0b00000101, 12, 34, 56]);
    }
}
