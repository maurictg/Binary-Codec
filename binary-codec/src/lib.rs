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
}

pub trait BinaryDeserializer<T: Clone = ()>: Sized {
    fn read_bytes(
        stream: &mut BitStreamReader,
        config: Option<&mut SerializerConfig<T>>,
    ) -> Result<Self, DeserializationError>;
}

mod bitstream;
mod config;
pub mod encoding;
pub mod utils;
pub mod variable;

pub use binary_codec_derive::{FromBytes, ToBytes};
pub use config::SerializerConfig;
pub use bitstream::{reader::BitStreamReader, writer::BitStreamWriter};
