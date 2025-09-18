use crate::{DeserializationError, SerializationError, SerializerConfig, utils::{ensure_size, get_read_size, slice, write_size}};

pub fn read_string(bytes: &[u8], size_key: Option<&str>, config: &mut SerializerConfig) -> Result<String, DeserializationError> {
    let len = get_read_size(bytes, size_key, config)?;
    config.reset_bits(true);
    let slice = slice(config, bytes, len, true)?;
    let string = String::from_utf8(slice.to_vec()).expect("Not valid UTF-8 bytes to create string");
    
    Ok(string)
}

pub fn write_string(value: &str, size_key: Option<&str>, buffer: &mut Vec<u8>, config: &mut SerializerConfig) -> Result<(), SerializationError> {
    config.reset_bits(false);
    write_size(value.len(), size_key, buffer, config)?; 

    buffer.extend_from_slice(&value.as_bytes());
    config.pos += value.len();
    Ok(())
}

pub fn read_object<T>(bytes: &[u8], size_key: Option<&str>, config: &mut SerializerConfig) -> Result<T, DeserializationError>  
    where T : crate::BinaryDeserializer 
{
    let len = get_read_size(bytes, size_key, config)?;

    // If exact size of buffer is available, don't slice
    if ensure_size(config, bytes, len)? {
        T::from_bytes(bytes, Some(config))
    } else {
        // Create an isolated slice like we do for a String, but with its own config
        config.reset_bits(true);
        let mut temp_config = config.clone();
        temp_config.reset();

        let slice = slice(config, bytes, len, true)?;
        T::from_bytes(&slice, Some(&mut temp_config))
    }
}

pub fn write_object<T>(value: &T, size_key: Option<&str>, buffer: &mut Vec<u8>, config: &mut SerializerConfig) -> Result<(), SerializationError>  
    where T : crate::BinarySerializer 
{
    // If length name is provided, we need to ensure the length matches
    // So we write it to a different buffer
    if size_key.is_some() {
        let mut temp_buffer = Vec::new();
        config.reset_bits(false);
        value.write_bytes(&mut temp_buffer, Some(config))?;
        write_size(temp_buffer.len(), size_key, buffer, config)?;
        buffer.extend_from_slice(&temp_buffer);
        Ok(())
    } else {
        value.write_bytes(buffer, Some(config))
    }
}

#[cfg(test)]
mod tests {
    use crate::{BinaryDeserializer, dynamics::read_small_dynamic_unsigned, fixed_int::FixedInt};

    use super::*;

    struct TestObj {
        nr: u16
    }

    impl BinaryDeserializer for TestObj {
        fn from_bytes(bytes: &[u8], config: Option<&mut SerializerConfig>) -> Result<Self, DeserializationError> {
            let config = config.unwrap();
            let nr = FixedInt::read(bytes, config)?;
            Ok(TestObj { nr })
        }
    }

    #[test]
    fn test_read_number_object_after_reading_few_bits() {
        let bytes = vec![0b0000_0011, 0, 7];
        let mut config = SerializerConfig::new();

        let small_nr = read_small_dynamic_unsigned(&bytes, &mut config, 2).unwrap();
        assert_eq!(small_nr, 3);
        assert_eq!(config.bits, 2);
        assert_eq!(config.pos, 0);

        let obj = read_object::<TestObj>(&bytes, None, &mut config).unwrap();
        assert_eq!(obj.nr, 7);
        assert_eq!(config.bits, 0);
        assert_eq!(config.pos, 3);
    }
}