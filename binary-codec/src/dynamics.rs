use crate::{DeserializationError, SerializationError, SerializerConfig, fixed_int::ZigZag};

pub fn read_small_dynamic_unsigned(
    bytes: &[u8],
    config: &mut SerializerConfig,
    bit_count: u8,
) -> Result<u8, DeserializationError> {
    read_small_dynamic(bytes, config, bit_count)
}

pub fn read_small_dynamic_signed(
    bytes: &[u8],
    config: &mut SerializerConfig,
    bit_count: u8,
) -> Result<i8, DeserializationError> {
    let val = read_small_dynamic(bytes, config, bit_count)?;
    Ok(ZigZag::to_signed(val))
}

pub fn write_small_dynamic_unsigned(
    val: u8,
    bytes: &mut Vec<u8>,
    config: &mut SerializerConfig,
    bit_count: u8,
) -> Result<(), SerializationError> {
    let max = (1u8 << bit_count) - 1;

    if val > max {
        return Err(SerializationError::ValueOutOfBounds(val as i32, 0, max as i32));
    }

    write_small_dynamic(val, bytes, config, bit_count)
}

pub fn write_small_dynamic_signed(
    val: i8,
    bytes: &mut Vec<u8>,
    config: &mut SerializerConfig,
    bit_count: u8,
) -> Result<(), SerializationError> {
    let min = -(1i8 << (bit_count - 1));
    let max = (1i8 << (bit_count - 1)) - 1;

    if val < min || val > max {
        return Err(SerializationError::ValueOutOfBounds(val as i32, min as i32, max as i32));
    }

    write_small_dynamic(val.to_unsigned(), bytes, config, bit_count)
}

pub fn write_bool(
    val: bool,
    bytes: &mut Vec<u8>,
    config: &mut SerializerConfig
) -> Result<(), SerializationError> {
    let val_u8 = if val { 1 } else { 0 };
    write_small_dynamic(val_u8, bytes, config, 1)
}

pub fn read_bool(
    bytes: &[u8],
    config: &mut SerializerConfig
) -> Result<bool, DeserializationError> {
    let val = read_small_dynamic(bytes, config, 1)?;
    Ok(val != 0)
}

fn create_mask(bits: &u8, bit_count: u8) -> u8 {
    let mask = (1u8 << bit_count) - 1u8;
    return mask << *bits;
}

fn read_small_dynamic(
    bytes: &[u8],
    config: &mut SerializerConfig,
    bit_count: u8
) -> Result<u8, DeserializationError>
{
    if config.bits == 8 || config.bits + bit_count > 8 {
        config.bits = 0;
        config.pos += 1;
    }

    let mask = create_mask(&config.bits, bit_count);

    let val = bytes[config.pos];
    let result = (val & mask) >> config.bits;
    config.bits += bit_count;

    Ok(result)
}

fn write_small_dynamic(
    val: u8,
    bytes: &mut Vec<u8>,
    config: &mut SerializerConfig,
    bit_count: u8
) -> Result<(), SerializationError>
{
    if config.bits == 0 || config.bits + bit_count > 8 {
        config.bits = 0;
        
        if bytes.len() > 0 {
            config.pos += 1;
        }

        bytes.push(0u8);
    }

    let mask = create_mask(&config.bits, bit_count);

    bytes[config.pos] &= !mask;
    bytes[config.pos] |= (val << config.bits) & mask;

    config.bits += bit_count;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_small_dynamic_unsigned() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        let bit_count = 4;
        let val: u8 = 0b1010;
        write_small_dynamic_unsigned(val, &mut bytes, &mut config, bit_count).unwrap();
        let mut config = SerializerConfig::new();
        let result = read_small_dynamic_unsigned(&bytes, &mut config, bit_count).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_write_read_small_dynamic_signed() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        let bit_count = 4;
        let val: i8 = -3;
        write_small_dynamic_signed(val, &mut bytes, &mut config, bit_count).unwrap();
        let mut config = SerializerConfig::new();
        let result = read_small_dynamic_signed(&bytes, &mut config, bit_count).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_write_read_small_dynamic_unsigned_existing_byte() {
        let mut bytes = vec![0b0000_0111]; // 7
        let mut config = SerializerConfig::new();
        config.pos = 0;
        config.bits = 4;
        let bit_count = 4;
        let val: u8 = 5;
        write_small_dynamic_unsigned(val, &mut bytes, &mut config, bit_count).unwrap();
        assert_eq!(bytes, vec![0b0101_0111]);
        let mut config = SerializerConfig::new();
        let result = read_small_dynamic_unsigned(&bytes, &mut config, bit_count).unwrap();
        assert_eq!(result, 7); // first 4 bits
        let result = read_small_dynamic_unsigned(&bytes, &mut config, bit_count).unwrap();
        assert_eq!(result, 5); // last 4 bits
    }

    #[test]
    fn test_write_small_dynamic_unsigned_out_of_bounds() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        let bit_count = 3;
        let val: u8 = 0b1000; // 8, out of bounds for 3 bits
        let result = write_small_dynamic_unsigned(val, &mut bytes, &mut config, bit_count);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_small_dynamic_signed_out_of_bounds() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        let bit_count = 3;
        let val: i8 = 5; // out of bounds for 3 bits signed
        let result = write_small_dynamic_signed(val, &mut bytes, &mut config, bit_count);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_read_bool_true() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        write_bool(true, &mut bytes, &mut config).unwrap();
        let mut config = SerializerConfig::new();
        let result = read_bool(&bytes, &mut config).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_write_read_bool_false() {
        let mut bytes = Vec::new();
        let mut config = SerializerConfig::new();
        write_bool(false, &mut bytes, &mut config).unwrap();
        let mut config = SerializerConfig::new();
        let result = read_bool(&bytes, &mut config).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_create_mask_3() {
        let bits = 2u8;
        let bit_count = 3u8;
        let mask = create_mask(&bits, bit_count);
        assert_eq!(mask, 0b0001_1100);
    }

    #[test]
    fn test_create_mask_2() {
        let bits = 3u8;
        let bit_count = 2u8;
        let mask = create_mask(&bits, bit_count);
        assert_eq!(mask, 0b0001_1000);
    }
}
