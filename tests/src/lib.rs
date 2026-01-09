use std::{cell::RefCell, collections::HashMap};

use binary_codec_derive::{FromBytes, ToBytes};
// mod out;

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct ExampleObject {
    #[toggles("toggle")]
    toggle: bool,

    #[toggled_by("toggle")]
    eventual1: Option<u32>,

    #[toggles("toggle2")]
    toggle2: bool,

    #[toggled_by("toggle2")]
    eventual2: Option<Nested>,

    #[length_for = "len"]
    #[length_for = "len2"]
    #[bits = 3]
    length: u8,

    bools: [bool; 3],

    #[length_by = "len"]
    array1: Vec<u8>,

    boollie: bool,

    #[length_by("len2")]
    str: String,

    #[dyn_length]
    dyn_len_arr: Vec<u16>,

    #[dyn_length]
    #[val_dyn_length]
    my_map: HashMap<u8, String>,

    #[dyn_int]
    dyn_int: u64,

    data: [u8; 16],
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum Nested {
    A(u32),
    B(u64),
    C,
    D { x: u32 },
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Testt {
    #[toggles("1")]
    has1: bool,
    #[toggles("2")]
    has2: bool,
    #[toggles("3")]
    has3: RefCell<bool>,
    #[toggles("bool")]
    boolean_is_true: bool,

    #[multi_enum]
    multi: Vec<MultiEnum>,

    #[multi_enum]
    boolean: Boolean,

    #[dyn_length]
    dyn_string: String,

    pub ref_val: RefCell<String>,
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
#[no_discriminator]
enum MultiEnum {
    #[toggled_by = "1"]
    Value1(u8),
    #[toggled_by = "2"]
    Value2(u16),
    #[toggled_by = "3"]
    Value3(u32),
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
#[no_discriminator]
enum Boolean {
    #[toggled_by = "!bool"]
    False(u8),
    #[toggled_by = "bool"]
    True(u16),
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum Flags {
    Wav,
    OptionOne {
        #[toggles("flaga")]
        flag_a: bool,

        #[toggles("flagb")]
        flag_b: bool,
    },
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
#[no_discriminator]
enum VariantByBool {
    False(u8),
    True(u8),
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum CustomDiscriminator {
    A = 3,
    B,
    C = 12,
    D,
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct ContainsBool {
    #[toggles("boolean")]
    flag: bool,

    #[variant_by = "boolean"]
    value: VariantByBool,
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct SliceArray {
    slices: Vec<[u8; 3]>,
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct ToggledByVariant {
    #[variant_for = "test"]
    num: u8,

    #[toggled_by_variant = "test=2|4|6"]
    even: Option<u8>,

    #[toggled_by_variant = "test=1|3|5"]
    uneven: Option<u16>
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use binary_codec::{BinaryDeserializer, BinarySerializer, SerializerConfig};

    use super::*;

    #[test]
    fn test_toggled_by_variant() {
        let t1 = ToggledByVariant {
            num: 1,
            even: None,
            uneven: Some(5)
        };

        let t2 = ToggledByVariant {
            num: 2,
            even: Some(5),
            uneven: None
        };

        let config: Option<&mut SerializerConfig> = None;
        let t1s = t1.to_bytes(config).unwrap();
        let config: Option<&mut SerializerConfig> = None;
        assert_eq!(ToggledByVariant::from_bytes(&t1s, config).unwrap(), t1);

        let config: Option<&mut SerializerConfig> = None;
        let t2s = t2.to_bytes(config).unwrap();
        let config: Option<&mut SerializerConfig> = None;
        assert_eq!(ToggledByVariant::from_bytes(&t2s, config).unwrap(), t2);
    }

    #[test]
    fn test_custom_discriminator() {
        let a = CustomDiscriminator::A;
        let b: CustomDiscriminator = CustomDiscriminator::B;
        let c = CustomDiscriminator::C;
        let d: CustomDiscriminator = CustomDiscriminator::D;
        let bytes_a = BinarySerializer::<()>::to_bytes(&a, None).unwrap();
        let bytes_b = BinarySerializer::<()>::to_bytes(&b, None).unwrap();
        let bytes_c: Vec<u8> = BinarySerializer::<()>::to_bytes(&c, None).unwrap();
        let bytes_d: Vec<u8> = BinarySerializer::<()>::to_bytes(&d, None).unwrap();

        assert_eq!(bytes_a, vec![3]);
        assert_eq!(bytes_b, vec![4]);
        assert_eq!(bytes_c, vec![12]);
        assert_eq!(bytes_d, vec![13]);
    }

    #[test]
    fn test_multi_slice() {
        let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let config: Option<&mut SerializerConfig> = None;
        let sa = SliceArray::from_bytes(&bytes, config).unwrap();
        assert_eq!(
            sa,
            SliceArray {
                slices: vec![[1, 2, 3], [4, 5, 6], [7, 8, 9]]
            }
        )
    }

    #[test]
    fn testenum() {
        let t = Flags::OptionOne {
            flag_a: true,
            flag_b: true,
        };
        let mut config = SerializerConfig::<()>::new(None);

        let bytes = BinarySerializer::to_bytes(&t, Some(&mut config)).unwrap();
        println!("{:?} [{}]", bytes, bytes.len());
        println!("{:?}", config);
    }

    #[test]
    fn multienum() {
        let t = Testt {
            has1: false,
            has2: true,
            has3: RefCell::new(true),
            boolean_is_true: true,
            boolean: Boolean::True(55),
            multi: vec![MultiEnum::Value2(34), MultiEnum::Value3(34)],
            dyn_string: String::from("Hello, world!"),
            ref_val: RefCell::new(String::from("hello!")),
        };

        let bytes = BinarySerializer::<()>::to_bytes(&t, None).unwrap();
        println!("{:?} [{}]", bytes, bytes.len());

        let t2 = BinaryDeserializer::<()>::from_bytes(&bytes, None).unwrap();

        assert_eq!(t, t2);
    }

    #[test]
    fn can_do_variant_by_bool() {
        let c = ContainsBool {
            flag: true,
            value: VariantByBool::True(5),
        };

        let c2 = ContainsBool {
            flag: false,
            value: VariantByBool::False(7),
        };

        let b1 = BinarySerializer::<()>::to_bytes(&c, None).unwrap();
        let b2 = BinarySerializer::<()>::to_bytes(&c2, None).unwrap();

        let c1d = BinaryDeserializer::<()>::from_bytes(&b1, None).unwrap();
        let c2d = BinaryDeserializer::<()>::from_bytes(&b2, None).unwrap();

        assert_eq!(c, c1d);
        assert_eq!(c2, c2d);
    }

    #[test]
    fn can_do_single_multienum() {
        #[derive(ToBytes, FromBytes, Debug, PartialEq)]
        #[no_discriminator]
        enum MyMultiEnum {
            #[toggled_by = "a"]
            A,
            #[toggled_by = "b"]
            B,
            #[toggled_by = "c"]
            C,
        }

        #[derive(ToBytes, FromBytes, Debug, PartialEq)]
        struct MyStruct {
            var: u8,

            #[variant_by = "variant"]
            mme: MyMultiEnum,
        }
    }

    #[test]
    fn it_works() {
        let o = ExampleObject {
            toggle: true,
            eventual1: Some(42),
            toggle2: true,
            eventual2: Some(Nested::B(12345)),
            length: 3,
            bools: [true, false, true],
            array1: vec![1, 2, 3],
            boollie: true,
            str: "Hoi".to_string(),
            dyn_len_arr: vec![3, 2, 1],
            dyn_int: 777777,
            my_map: {
                let mut m = HashMap::new();
                m.insert(1, String::from("hello"));
                m.insert(2, String::from("world!"));
                m
            },
            data: [0u8; 16],
        };

        let x = Nested::D { x: 5 };
        let discr: u8 = x.get_discriminator();

        println!("DISC: {}", discr);

        let bytes = BinarySerializer::<()>::to_bytes(&o, None).unwrap();
        println!("{:?} [{}]", bytes, bytes.len());

        let o2 = BinaryDeserializer::<()>::from_bytes(&bytes, None).unwrap();

        assert_eq!(o, o2);
    }
}
