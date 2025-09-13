#[derive(Debug)]
pub enum SerializationError {
    /// Value is out of bounds (value, min, max)
    ValueOutOfBounds(i32, i32, i32),

    // Unexpected size (expected, actual)
    UnexpectedLength(usize, usize)
}

#[derive(Debug)]
pub enum DeserializationError {
    /// Not enough bytes (bytes missing)
    NotEnoughBytes(usize),

    /// Unknown enum discriminator
    UnknownDiscriminant(u8)
}

pub struct SerializationConfig {
    pub strict_mode: bool
}

mod tests;
pub mod serializers;
pub mod encodings;
pub mod dyn_int;
pub use binary_codec_derive::{ToBytes, FromBytes};