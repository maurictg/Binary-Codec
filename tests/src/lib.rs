use binary_codec_derive::{FromBytes, ToBytes};
use binary_codec::BinarySerializer;
use binary_codec::BinaryDeserializer;

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct ExampleObject {
    #[toggle_key = "toggle"]
    toggle: bool,

    #[toggled_by_key = "toggle"]
    eventual1: Option<u32>,

    toggle2: bool,

    #[toggled_by = "toggle2"]
    eventual2: Option<Nested>,

    #[length_key = "len"]
    #[bits = 3]
    length: u8,

    bools: [bool; 3],

    #[length_determined_by = "length"]
    array1: Vec<u8>,

    boollie: bool,

    #[length_by_key = "len"]
    str: String
}

#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum Nested {
    A(u32),
    B(u64),
}


#[cfg(test)]
mod tests {
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
            str: "Hoi".to_string()
        };

        let bytes = o.to_bytes(None).unwrap();
        let o2 = ExampleObject::from_bytes(&bytes, None).unwrap();

        println!("{:?} [{}]", bytes, bytes.len());
        assert_eq!(o, o2);
    }
}
