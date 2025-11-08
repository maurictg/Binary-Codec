use std::collections::HashMap;

use binary_codec_derive::{FromBytes, ToBytes};
use binary_codec::BinarySerializer;
use binary_codec::BinaryDeserializer;
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
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum Nested {
    A(u32),
    B(u64),
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use binary_codec::SerializerConfig;

    use super::*;

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
            }
        };

        let x = Nested::B(5);
        let discr: u8 = x.get_discriminator();

        println!("DISC: {}", discr);

        let bytes = o.to_bytes(None).unwrap();
        println!("{:?} [{}]", bytes, bytes.len());

        let o2 = ExampleObject::from_bytes(&bytes, None).unwrap();

        assert_eq!(o, o2);
    }
}
