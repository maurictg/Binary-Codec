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
    fn serialize_bytes(
        &self,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Vec<u8>, SerializationError>;
    fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        self.serialize_bytes(None)
    }

    fn write_bytes(
        &self,
        buffer: &mut Vec<u8>,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<(), SerializationError>;
}

pub trait BinaryDeserializer<T: Clone = ()>: Sized {
    fn deserialize_bytes(
        bytes: &[u8],
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Self, DeserializationError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError> {
        Self::deserialize_bytes(bytes, None)
    }
}

pub mod bitstream;
mod config;
pub mod dynamics;
pub mod encoding;
pub mod utils;
pub mod variable;

pub use binary_codec_derive::{FromBytes, ToBytes};
pub use config::SerializerConfig;
