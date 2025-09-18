use std::{collections::HashMap};

#[derive(Clone, Debug)]
pub struct SerializerConfig {
    toggle_keys: HashMap<String, bool>,
    length_keys: HashMap<String, usize>,
    variant_keys: HashMap<String, u8>,
    pub bits: u8,
    pub pos: usize,
    pub discriminator: Option<u8>
}

impl SerializerConfig {
    pub fn new() -> Self {
        Self {
            toggle_keys: HashMap::new(),
            length_keys: HashMap::new(),
            variant_keys: HashMap::new(),
            bits: 0,
            pos: 0,
            discriminator: None
        }
    }

    pub fn next_reset_bits_pos(&self) -> usize {
        if self.bits == 0 {
            self.pos
        } else {
            self.pos + 1
        }
    }

    pub fn reset_bits(&mut self, is_read: bool) {
        if self.bits != 0 && is_read {
            self.pos += 1;
        }
        self.bits = 0;
    }

    pub fn set_toggle(&mut self, key: &str, value: bool) {
        println!("Setting toggle key {} to {}", key, value);
        self.toggle_keys.insert(key.to_string(), value);
    }

    pub fn set_length(&mut self, key: &str, value: usize) {
        self.length_keys.insert(key.to_string(), value);
    }

    pub fn set_variant(&mut self, key: &str, value: u8) {
        self.variant_keys.insert(key.to_string(), value);
    }

    pub fn get_toggle(&self, key: &str) -> Option<bool> {
        if key.starts_with('!') {
            let key = &key[1..];
            return self.toggle_keys.get(key).map(|v| !*v);
        }
        
        self.toggle_keys.get(key).copied()
    }

    pub fn get_length(&self, key: &str) -> Option<usize> {
        self.length_keys.get(key).copied()
    }

    pub fn get_variant(&self, key: &str) -> Option<u8> {
        self.variant_keys.get(key).copied()
    }

    pub fn reset(&mut self) {
        self.bits = 0;
        self.pos = 0;
        self.discriminator = None;
    }
}