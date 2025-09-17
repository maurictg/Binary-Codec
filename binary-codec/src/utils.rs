use crate::{DeserializationError, SerializationError, SerializerConfig, dyn_int::{read_dynint, write_dynint}};

pub fn ensure_size(config: &SerializerConfig, bytes: &[u8], required: usize) -> Result<bool, DeserializationError> {
    if config.pos + required > bytes.len() {
        return Err(DeserializationError::NotEnoughBytes(config.pos + required - bytes.len()));
    }
    Ok(config.pos + required == bytes.len())
}

pub fn slice<'a>(config: &mut SerializerConfig, bytes: &'a [u8], length: usize, increment: bool) -> Result<&'a [u8], DeserializationError> {
    ensure_size(config, bytes, length)?;
    let slice = &bytes[config.pos..config.pos + length];
    if increment {
        config.pos += length;
    }
    Ok(slice)
}

pub fn get_read_size<'a>(bytes: &'a [u8], size_key: Option<&str>, config: &mut SerializerConfig) -> Result<usize, DeserializationError> {
    let size = if let Some(size_key) = size_key {
        // Special case: dynamic length prefix
        if size_key == "__dynamic" {
            return read_dynint(bytes, config).map(|v| v as usize);
        }

        config.get_length(size_key).unwrap_or(bytes.len() - config.pos)
    } else {
        bytes.len() - config.pos
    };

    ensure_size(config, bytes, size)?;
    Ok(size)
}

pub fn write_size(size: usize, size_key: Option<&str>, buffer: &mut Vec<u8>, config: &mut SerializerConfig) -> Result<(), SerializationError> {
    if let Some(size_key) = size_key {
        // Special case: dynamic length prefix
        if size_key == "__dynamic" {
            return write_dynint(size as u128, buffer, config);
        }

        if let Some(expected) = config.get_length(size_key) {
            if expected != size {
                return Err(SerializationError::UnexpectedLength(expected, size));
            }
        }
    }

    Ok(())
}