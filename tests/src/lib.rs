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
    #[bits = 3]
    length: u8,

    bools: [bool; 3],

    #[length_by = "len"]
    array1: Vec<u8>,

    boollie: bool,

    #[length_by("len")]
    str: String,

    #[dyn_length]
    dyn_len_arr: Vec<u16>,

    #[dyn_length]
    #[val_dyn_length]
    my_map: HashMap<u8, String>,

    #[dyn_int]
    dyn_int: u64,

    data: [u8; 16]
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use binary_codec::{BinaryDeserializer, BinarySerializer, SerializerConfig};

    use super::*;

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
            ref_val: RefCell::new(String::from("hello!")),
        };

        let bytes = BinarySerializer::<()>::to_bytes(&t, None).unwrap();
        println!("{:?} [{}]", bytes, bytes.len());

        let t2 = BinaryDeserializer::<()>::from_bytes(&bytes, None).unwrap();

        assert_eq!(t, t2);
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
