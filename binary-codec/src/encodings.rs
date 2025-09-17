use crate::{DeserializationError, SerializationError};

pub fn write_zigzag<T, const S: usize>(
    val: T,
    bytes: &mut Vec<u8>,
    pos: &mut usize,
    bits: &mut u8,
) -> Result<(), SerializationError>
where
    T: ZigZag,
    T::Unsigned: FixedInt<S>,
{
    let encoded = val.to_unsigned();
    encoded.write(bytes, pos, bits)
}

pub fn read_zigzag<T, const S: usize>(
    bytes: &[u8],
    pos: &mut usize,
    bits: &mut u8,
) -> Result<T, DeserializationError>
where
    T: ZigZag,
    T::Unsigned: FixedInt<S>,
{
    let raw = T::Unsigned::read(bytes, pos, bits)?;
    Ok(T::to_signed(raw))
}

// Fixed int implementations
pub trait FixedInt<const S: usize> : Sized {    
    fn serialize(self) -> [u8; S];
    fn deserialize(bytes: &[u8]) -> Self;

    fn write(
        self,
        bytes: &mut Vec<u8>,
        pos: &mut usize,
        bits: &mut u8,
    ) -> Result<(), SerializationError> {
        *bits = 0;
        bytes.extend_from_slice(&self.serialize());
        *pos += S;
        Ok(())
    }

    fn read(
        bytes: &[u8],
        pos: &mut usize,
        bits: &mut u8,
    ) -> Result<Self, DeserializationError> {
        // "reset_bits"
        if *bits != 0 && *pos == 0 {
            *pos += 1;
        }

        *bits = 0;
        
        if *pos + S > bytes.len() {
            return Err(DeserializationError::NotEnoughBytes(*pos + S - (bytes.len() - 1)));
        }

        let val = Self::deserialize(&bytes[*pos..*pos + S]);
        *pos += S;
        Ok(val)
    }
}

impl FixedInt<1> for u8 {
    fn serialize(self) -> [u8; 1] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u8::from_be_bytes(bytes.try_into().unwrap()).try_into().unwrap()
    }
}

impl FixedInt<2> for u16 {
    fn serialize(self) -> [u8; 2] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u16::from_be_bytes(bytes.try_into().unwrap()).try_into().unwrap()
    }
}

impl FixedInt<4> for u32 {
    fn serialize(self) -> [u8; 4] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u32::from_be_bytes(bytes.try_into().unwrap()).try_into().unwrap()
    }
}

impl FixedInt<8> for u64 {
    fn serialize(self) -> [u8; 8] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u64::from_be_bytes(bytes.try_into().unwrap()).try_into().unwrap()
    }
}

impl FixedInt<16> for u128 {
    fn serialize(self) -> [u8; 16] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u128::from_be_bytes(bytes.try_into().unwrap()).try_into().unwrap()
    }
}

// ZigZag implementations
pub trait ZigZag {
    type Unsigned;

    fn to_unsigned(self) -> Self::Unsigned;
    fn to_signed(n: Self::Unsigned) -> Self;
}

impl ZigZag for i8 {
    type Unsigned = u8;
    fn to_unsigned(self) -> u8 {
        ((self << 1) ^ (self >> 7)) as u8
    }
    fn to_signed(n: u8) -> i8 {
        ((n >> 1) as i8) ^ -((n & 1) as i8)
    }
}

impl ZigZag for i16 {
    type Unsigned = u16;
    fn to_unsigned(self) -> u16 {
        ((self << 1) ^ (self >> 15)) as u16
    }
    fn to_signed(n: u16) -> i16 {
        ((n >> 1) as i16) ^ -((n & 1) as i16)
    }
}

impl ZigZag for i32 {
    type Unsigned = u32;
    fn to_unsigned(self) -> u32 {
        ((self << 1) ^ (self >> 31)) as u32
    }
    fn to_signed(n: u32) -> i32 {
        ((n >> 1) as i32) ^ -((n & 1) as i32)
    }
}

impl ZigZag for i64 {
    type Unsigned = u64;
    fn to_unsigned(self) -> u64 {
        ((self << 1) ^ (self >> 63)) as u64
    }
    fn to_signed(n: u64) -> i64 {
        ((n >> 1) as i64) ^ -((n & 1) as i64)
    }
}

impl ZigZag for i128 {
    type Unsigned = u128;
    fn to_unsigned(self) -> u128 {
        ((self << 1) ^ (self >> 127)) as u128
    }
    fn to_signed(n: u128) -> i128 {
        ((n >> 1) as i128) ^ -((n & 1) as i128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! zigzag_test {
        ($name:ident, $ty:ty, $unsigned:ty, $val:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let val: $ty = $val;
                let encoded: $unsigned = val.to_unsigned();
                assert_eq!(encoded, $expected, "ZigZag encoding failed for {}", val);
                let decoded: $ty = <$ty>::to_signed(encoded);
                assert_eq!(decoded, val, "ZigZag decoding failed for {}", val);
            }
        };
    }

    zigzag_test!(zigzag_i16_pos, i16, u16, 0b0000_0000_0000_0010, 0b0000_0000_0000_0100);
    zigzag_test!(zigzag_i16_neg, i16, u16, -0b0000_0000_0000_0010, 0b0000_0000_0000_0011);
    zigzag_test!(zigzag_i32_pos, i32, u32, 0b0000_0000_0000_0000_0000_0000_0000_0010, 0b0000_0000_0000_0000_0000_0000_0000_0100);
    zigzag_test!(zigzag_i32_neg, i32, u32, -0b0000_0000_0000_0000_0000_0000_0000_0010, 0b0000_0000_0000_0000_0000_0000_0000_0011);
    zigzag_test!(zigzag_i64_pos, i64, u64, 0b10, 0b100);
    zigzag_test!(zigzag_i64_neg, i64, u64, -0b10, 0b11);
    zigzag_test!(zigzag_i128_pos, i128, u128, 0b10, 0b100);
    zigzag_test!(zigzag_i128_neg, i128, u128, -0b10, 0b11);

    macro_rules! fixedint_test {
        ($name:ident, $ty:ty, $val:expr, $bytes:expr) => {
            #[test]
            fn $name() {
                let val: $ty = $val;
                let serialized = val.serialize();
                assert_eq!(serialized, $bytes, "FixedInt serialize failed for {}", val);
                let deserialized = <$ty>::deserialize(&serialized);
                assert_eq!(deserialized, val, "FixedInt deserialize failed for {:?}", serialized);
            }
        };
    }

    fixedint_test!(fixedint_u16, u16, 0b1010_1010_1010_1010, [0b1010_1010; 2]);
    fixedint_test!(fixedint_u32, u32, 0b1010_1010_1010_1010_1010_1010_1010_1010, [0b1010_1010; 4]);
    fixedint_test!(fixedint_u64, u64, 0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010, [0b1010_1010; 8]);
    fixedint_test!(fixedint_u128, u128, 0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010, [0b1010_1010; 16]);

    #[test]
    fn test_write_read_zigzag_i32() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let val: i32 = -123;
        write_zigzag::<i32, 4>(val, &mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let decoded = read_zigzag::<i32, 4>(&bytes, &mut pos, &mut bits).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_write_read_zigzag_i64() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let val: i64 = 456789;
        write_zigzag::<i64, 8>(val, &mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let decoded = read_zigzag::<i64, 8>(&bytes, &mut pos, &mut bits).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_write_read_fixedint_u32() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let val: u32 = 0b1010_1010_1010_1010_1010_1010_1010_1010;
        val.write(&mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let decoded = u32::read(&bytes, &mut pos, &mut bits).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_write_read_fixedint_u128() {
        let mut bytes = Vec::new();
        let mut pos = 0;
        let mut bits = 0;
        let val: u128 = 0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010;
        val.write(&mut bytes, &mut pos, &mut bits).unwrap();
        pos = 0;
        bits = 0;
        let decoded = u128::read(&bytes, &mut pos, &mut bits).unwrap();
        assert_eq!(decoded, val);
    }
}