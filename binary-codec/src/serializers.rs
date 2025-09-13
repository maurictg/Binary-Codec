use crate::{encodings::ZigZag, DeserializationError, SerializationError};

pub fn read_small_dynamic_unsigned(
    bytes: &[u8],
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8,
) -> Result<u8, DeserializationError> {
    read_small_dynamic(bytes, pos, bits, bit_count)
}

pub fn read_small_dynamic_signed(
    bytes: &[u8],
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8,
) -> Result<i8, DeserializationError> {
    let val = read_small_dynamic(bytes, pos, bits, bit_count)?;
    Ok(ZigZag::to_signed(val))
}

pub fn write_small_dynamic_unsigned(
    val: u8,
    bytes: &mut Vec<u8>,
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8,
) -> Result<(), SerializationError> {
    let max = (1u8 << bit_count) - 1;

    if val > max {
        return Err(SerializationError::ValueOutOfBounds(val as i32, 0, max as i32));
    }

    write_small_dynamic(val, bytes, pos, bits, bit_count)
}

pub fn write_small_dynamic_signed(
    val: i8,
    bytes: &mut Vec<u8>,
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8,
) -> Result<(), SerializationError> {
    let min = -(1i8 << (bit_count - 1));
    let max = (1i8 << (bit_count - 1)) - 1;

    if val < min || val > max {
        return Err(SerializationError::ValueOutOfBounds(val as i32, min as i32, max as i32));
    }

    write_small_dynamic(val.to_unsigned(), bytes, pos, bits, bit_count)
}

pub fn write_bool(
    val: bool,
    bytes: &mut Vec<u8>,
    pos: &mut usize,
    bits: &mut u8
) -> Result<(), SerializationError> {
    let val_u8 = if val { 1 } else { 0 };
    write_small_dynamic(val_u8, bytes, pos, bits, 1)
}

pub fn read_bool(
    bytes: &[u8],
    pos: &mut usize,
    bits: &mut u8
) -> Result<bool, DeserializationError> {
    let val = read_small_dynamic(bytes, pos, bits, 1)?;
    Ok(val != 0)
}

fn create_mask(bits: &u8, bit_count: u8) -> u8 {
    let mask = (1u8 << bit_count) - 1u8;
    return mask << *bits;
}

// returns (bits, next_bits, next_byte)
fn next_bits_and_byte(bits: u8, bits_needed: u8) -> (u8, u8, bool) {
    let next_bits = bits + bits_needed;
    if next_bits > 8 {
        (0, bits_needed, true)
    } else if next_bits < 8 {
        (bits, next_bits, bits == 0)
    } else {
        (bits, 0, false)
    }
}

fn read_small_dynamic(
    bytes: &[u8],
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8
) -> Result<u8, DeserializationError>
{
    let (current_bits, next_bits, next_byte) = next_bits_and_byte(*bits, bit_count);
    let mask = create_mask(&current_bits, bit_count);

    if next_byte {
        *pos += 1;
    }

    let read_pos = if *pos == 0 { 0 } else { *pos - 1 };

    let val = bytes[read_pos];
    let result = (val & mask) >> current_bits;

    *bits = next_bits;
    Ok(result)
}

fn write_small_dynamic(
    val: u8,
    bytes: &mut Vec<u8>,
    pos: &mut usize,
    bits: &mut u8,
    bit_count: u8
) -> Result<(), SerializationError>
{
    let (current_bits, next_bits, next_byte) = next_bits_and_byte(*bits, bit_count);
    let mask = create_mask(&current_bits, bit_count);

    if next_byte {
        bytes.push(0u8);
        *pos += 1;
    }

    bytes[*pos - 1] &= !mask;
    bytes[*pos - 1] |= (val << current_bits) & mask;

    *bits = next_bits;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_small_dynamic_unsigned() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let bit_count = 4;
        let val: u8 = 0b1010;
        write_small_dynamic_unsigned(val, &mut bytes, &mut pos, &mut bits, bit_count).unwrap();
        pos = 0;
        bits = 0;
        let result = read_small_dynamic_unsigned(&bytes, &mut pos, &mut bits, bit_count).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_write_read_small_dynamic_signed() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let bit_count = 4;
        let val: i8 = -3;
        write_small_dynamic_signed(val, &mut bytes, &mut pos, &mut bits, bit_count).unwrap();
        pos = 0;
        bits = 0;
        let result = read_small_dynamic_signed(&bytes, &mut pos, &mut bits, bit_count).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_write_read_small_dynamic_unsigned_existing_byte() {
        let mut bytes = vec![0b0000_0111]; // 7
        let mut pos = 1;
        let mut bits = 4;
        let bit_count = 4;
        let val: u8 = 5;
        write_small_dynamic_unsigned(val, &mut bytes, &mut pos, &mut bits, bit_count).unwrap();
        assert_eq!(bytes, vec![0b0101_0111]);
        pos = 0;
        bits = 0;
        let result = read_small_dynamic_unsigned(&bytes, &mut pos, &mut bits, bit_count).unwrap();
        assert_eq!(result, 7); // first 4 bits
        let result = read_small_dynamic_unsigned(&bytes, &mut pos, &mut bits, bit_count).unwrap();
        assert_eq!(result, 5); // last 4 bits
    }

    #[test]
    fn test_write_small_dynamic_unsigned_out_of_bounds() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let bit_count = 3;
        let val: u8 = 0b1000; // 8, out of bounds for 3 bits
        let result = write_small_dynamic_unsigned(val, &mut bytes, &mut pos, &mut bits, bit_count);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_small_dynamic_signed_out_of_bounds() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let bit_count = 3;
        let val: i8 = 5; // out of bounds for 3 bits signed
        let result = write_small_dynamic_signed(val, &mut bytes, &mut pos, &mut bits, bit_count);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_read_bool_true() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        write_bool(true, &mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let result = read_bool(&bytes, &mut pos, &mut bits).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_write_read_bool_false() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        write_bool(false, &mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let result = read_bool(&bytes, &mut pos, &mut bits).unwrap();
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

    #[test]
    fn test_next_bits_and_byte() {
        let (bits, next_bits, next_byte) = next_bits_and_byte(6, 3);
        assert_eq!((bits, next_bits, next_byte), (0, 3, true));
        let (bits, next_bits, next_byte) = next_bits_and_byte(0, 3);
        assert_eq!((bits, next_bits, next_byte), (0, 3, true));
           let (bits, next_bits, next_byte) = next_bits_and_byte(8, 1);
        assert_eq!((bits, next_bits, next_byte), (0, 1, true));
        let (bits, next_bits, next_byte) = next_bits_and_byte(4, 4);
        assert_eq!((bits, next_bits, next_byte), (4, 0, false));
    }
}
