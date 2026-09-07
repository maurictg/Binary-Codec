use crate::encoding::zigzag::ZigZag;

/// FixedInt
pub trait FixedInt<const S: usize>: Sized {
    /// Serialize self to fixed-size big-endian u8 array
    fn serialize(self) -> [u8; S];

    /// Deserialize from big-endian slice
    fn deserialize(bytes: &[u8]) -> Self;
}

impl FixedInt<1> for u8 {
    fn serialize(self) -> [u8; 1] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u8::from_be_bytes(bytes.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl FixedInt<1> for i8 {
    fn serialize(self) -> [u8; 1] {
        FixedInt::serialize(self.to_unsigned())
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ZigZag::to_signed(FixedInt::deserialize(bytes))
    }
}

impl FixedInt<2> for u16 {
    fn serialize(self) -> [u8; 2] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u16::from_be_bytes(bytes.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl FixedInt<2> for i16 {
    fn serialize(self) -> [u8; 2] {
        FixedInt::serialize(self.to_unsigned())
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ZigZag::to_signed(FixedInt::deserialize(bytes))
    }
}

impl FixedInt<4> for u32 {
    fn serialize(self) -> [u8; 4] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u32::from_be_bytes(bytes.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl FixedInt<4> for i32 {
    fn serialize(self) -> [u8; 4] {
        FixedInt::serialize(self.to_unsigned())
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ZigZag::to_signed(FixedInt::deserialize(bytes))
    }
}

impl FixedInt<8> for u64 {
    fn serialize(self) -> [u8; 8] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u64::from_be_bytes(bytes.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl FixedInt<8> for i64 {
    fn serialize(self) -> [u8; 8] {
        FixedInt::serialize(self.to_unsigned())
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ZigZag::to_signed(FixedInt::deserialize(bytes))
    }
}

impl FixedInt<16> for u128 {
    fn serialize(self) -> [u8; 16] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        u128::from_be_bytes(bytes.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl FixedInt<16> for i128 {
    fn serialize(self) -> [u8; 16] {
        FixedInt::serialize(self.to_unsigned())
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ZigZag::to_signed(FixedInt::deserialize(bytes))
    }
}

impl FixedInt<4> for f32 {
    fn serialize(self) -> [u8; 4] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        f32::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl FixedInt<8> for f64 {
    fn serialize(self) -> [u8; 8] {
        self.to_be_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        f64::from_be_bytes(bytes.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::fixed_int::FixedInt;

    macro_rules! fixedint_test {
        ($name:ident, $ty:ty, $val:expr, $bytes:expr) => {
            #[test]
            fn $name() {
                let val: $ty = $val;
                let serialized = val.serialize();
                assert_eq!(serialized, $bytes, "FixedInt serialize failed for {}", val);
                let deserialized = <$ty>::deserialize(&serialized);
                assert_eq!(
                    deserialized, val,
                    "FixedInt deserialize failed for {:?}",
                    serialized
                );
            }
        };
    }

    fixedint_test!(fixedint_u16, u16, 0b1010_1010_1010_1010, [0b1010_1010; 2]);
    fixedint_test!(
        fixedint_u32,
        u32,
        0b1010_1010_1010_1010_1010_1010_1010_1010,
        [0b1010_1010; 4]
    );
    fixedint_test!(
        fixedint_u64,
        u64,
        0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010,
        [0b1010_1010; 8]
    );
    fixedint_test!(fixedint_u128, u128, 0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010, [0b1010_1010; 16]);
}
