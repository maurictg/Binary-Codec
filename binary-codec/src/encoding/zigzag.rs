// ZigZag implementations
pub trait ZigZag {
    type Unsigned;

    // Convert signed number to unsigned number with ZigZag encoding
    fn to_unsigned(self) -> Self::Unsigned;

    // Convert unsigned number back to signed number with ZigZag encoding
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

    zigzag_test!(zigzag_i8_pos, i8, u8, 0b0000_0010, 0b0000_0100);
    zigzag_test!(zigzag_i8_neg, i8, u8, -0b0000_0010, 0b0000_0011);

    zigzag_test!(
        zigzag_i16_pos,
        i16,
        u16,
        0b0000_0000_0000_0010,
        0b0000_0000_0000_0100
    );
    zigzag_test!(
        zigzag_i16_neg,
        i16,
        u16,
        -0b0000_0000_0000_0010,
        0b0000_0000_0000_0011
    );
    zigzag_test!(
        zigzag_i32_pos,
        i32,
        u32,
        0b0000_0000_0000_0000_0000_0000_0000_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0100
    );
    zigzag_test!(
        zigzag_i32_neg,
        i32,
        u32,
        -0b0000_0000_0000_0000_0000_0000_0000_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0011
    );
    zigzag_test!(zigzag_i64_pos, i64, u64, 0b10, 0b100);
    zigzag_test!(zigzag_i64_neg, i64, u64, -0b10, 0b11);
    zigzag_test!(zigzag_i128_pos, i128, u128, 0b10, 0b100);
    zigzag_test!(zigzag_i128_neg, i128, u128, -0b10, 0b11);

    #[test]
    fn we_actually_use_zigzag_encoding() {
        assert_eq!(39, ZigZag::to_unsigned(-20i8));
        assert_eq!(-2i8, ZigZag::to_signed(3u8));
        assert_eq!(0, ZigZag::to_unsigned(0i32));
        assert_eq!(20i32, ZigZag::to_signed(40u32));
    }
}
