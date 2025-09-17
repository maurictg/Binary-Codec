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

pub struct SerializerConfig {
    toggle_keys: HashMap<String, bool>,
    length_keys: HashMap<String, usize>
}

impl SerializerConfig {
    pub fn new() -> Self {
        Self {
            toggle_keys: HashMap::new(),
            length_keys: HashMap::new()
        }
    }

    pub fn set_toggle(&mut self, key: &str, value: bool) {
        println!("Setting toggle key {} to {}", key, value);
        self.toggle_keys.insert(key.to_string(), value);
    }

    pub fn set_length(&mut self, key: &str, value: usize) {
        self.length_keys.insert(key.to_string(), value);
    }

    pub fn get_toggle(&self, key: &str) -> Option<bool> {
        self.toggle_keys.get(key).copied()
    }

    pub fn get_length(&self, key: &str) -> Option<usize> {
        self.length_keys.get(key).copied()
    }
}

pub trait BinarySerializer {
    fn to_bytes(&self, config: Option<SerializerConfig>) -> Result<Vec<u8>, SerializationError>;
}

pub trait BinaryDeserializer : Sized {
    fn from_bytes(bytes: &[u8], config: Option<SerializerConfig>) -> Result<Self, DeserializationError>;
}

pub mod serializers;
pub mod encodings;
pub mod dyn_int;
use std::collections::HashMap;

pub use binary_codec_derive::{ToBytes, FromBytes};