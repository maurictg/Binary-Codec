#[derive(Debug, PartialEq)]
pub enum SerializationError {
    /// Value is out of bounds (value, min, max)
    ValueOutOfBounds(i32, i32, i32),

    // Unexpected size (expected, actual)
    UnexpectedLength(usize, usize),

    /// Missing runtime length key
    MissingLengthByKey(String),

    /// Validation did fail for the data
    InvalidData(String),
}

#[derive(Debug, PartialEq)]
pub enum DeserializationError {
    /// Not enough bytes (bytes missing)
    NotEnoughBytes(usize),

    // Unexpected size (expected, actual)
    UnexpectedLength(usize, usize),

    /// Unknown enum discriminator
    UnknownDiscriminant(u8),

    /// Missing runtime length key
    MissingLengthByKey(String),

    /// Validation did fail for the data
    InvalidData(String),
}

pub trait BinarySerializer<T: Clone = ()> {
    fn write_bytes(
        &self,
        stream: &mut BitStreamWriter,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<(), SerializationError>;

    fn to_bytes(
        &self,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Vec<u8>, SerializationError> {
        let mut buffer = Vec::new();
        let mut stream = BitStreamWriter::new(&mut buffer);
        self.write_bytes(&mut stream, config)?;
        Ok(buffer)
    }
}

pub trait BinaryDeserializer<T: Clone = ()>: Sized {
    fn read_bytes(
        stream: &mut BitStreamReader,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Self, DeserializationError>;

    fn from_bytes(
        bytes: &[u8],
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Self, DeserializationError> {
        let mut stream = BitStreamReader::new(bytes);
        Self::read_bytes(&mut stream, config)
    }
}

mod bitstream;
mod config;
pub mod encoding;
pub mod utils;

pub use encoding::zigzag::ZigZag;
pub use config::SerializerConfig;
pub use bitstream::{reader::BitStreamReader, writer::BitStreamWriter};
pub use binary_codec_derive::{FromBytes, ToBytes};

