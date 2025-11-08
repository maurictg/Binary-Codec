#[derive(Debug)]
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

#[derive(Debug)]
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

pub trait BinarySerializer {
    fn to_bytes(&self, config: Option<&mut SerializerConfig>) -> Result<Vec<u8>, SerializationError>;
    fn write_bytes(&self, buffer: &mut Vec<u8>, config: Option<&mut SerializerConfig>) -> Result<(), SerializationError>;
}

pub trait BinaryDeserializer : Sized {
    fn from_bytes(bytes: &[u8], config: Option<&mut SerializerConfig>) -> Result<Self, DeserializationError>;
}

mod config;
pub mod utils;
pub mod dynamics;
pub mod fixed_int;
pub mod dyn_int;
pub mod variable;

pub use binary_codec_derive::{ToBytes, FromBytes};
pub use config::SerializerConfig;