#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use std::collections::HashMap;
use binary_codec_derive::{FromBytes, ToBytes};
use binary_codec::BinarySerializer;
use binary_codec::BinaryDeserializer;
struct ExampleObject {
    #[toggle_key = "toggle"]
    toggle: bool,
    #[toggled_by = "toggle"]
    eventual1: Option<u32>,
    #[toggle_key = "toggle2"]
    toggle2: bool,
    #[toggled_by = "toggle2"]
    eventual2: Option<Nested>,
    #[length_key = "len"]
    #[bits = 3]
    length: u8,
    bools: [bool; 3],
    #[length_by = "len"]
    array1: Vec<u8>,
    boollie: bool,
    #[length_by = "len"]
    str: String,
    my_map: HashMap<u8, i32>,
}
impl binary_codec::BinarySerializer for ExampleObject {
    fn to_bytes(
        &self,
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<Vec<u8>, binary_codec::SerializationError> {
        let mut bytes = Vec::new();
        Self::write_bytes(self, &mut bytes, config)?;
        Ok(bytes)
    }
    fn write_bytes(
        &self,
        buffer: &mut Vec<u8>,
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<(), binary_codec::SerializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new();
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = buffer;
        let _p_val = &self.toggle;
        _p_config.set_toggle("toggle", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.eventual1;
        if _p_config.get_toggle("toggle").unwrap_or(false) {
            let _p_val = _p_val
                .as_ref()
                .expect("Expected Some value, because toggled_by field is true");
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
        }
        let _p_val = &self.toggle2;
        _p_config.set_toggle("toggle2", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.eventual2;
        if _p_config.get_toggle("toggle2").unwrap_or(false) {
            let _p_val = _p_val
                .as_ref()
                .expect("Expected Some value, because toggled_by field is true");
            binary_codec::variable::write_object(_p_val, None, _p_bytes, _p_config)?;
        }
        let _p_val = &self.length;
        _p_config.set_length("len", *_p_val as usize);
        binary_codec::dynamics::write_small_dynamic_unsigned(
            *_p_val,
            _p_bytes,
            _p_config,
            3u8,
        )?;
        let _p_val = &self.bools;
        for _p_val in _p_val {
            binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        }
        let _p_val = &self.array1;
        let _p_len = _p_val.len();
        binary_codec::utils::write_size(_p_len, Some("len"), _p_bytes, _p_config)?;
        for _p_val in _p_val {
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
        }
        let _p_val = &self.boollie;
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.str;
        binary_codec::variable::write_string(_p_val, Some("len"), _p_bytes, _p_config)?;
        let _p_val = &self.my_map;
        for (key, value) in _p_val {
            let _p_val = key;
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            let _p_val = value;
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
        }
        Ok(())
    }
}
impl binary_codec::BinaryDeserializer for ExampleObject {
    fn from_bytes(
        bytes: &[u8],
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<Self, binary_codec::DeserializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new();
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = bytes;
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let toggle = _p_val;
        _p_config.set_toggle("toggle", _p_val);
        let mut __option_0: Option<u32> = None;
        if _p_config.get_toggle("toggle").unwrap_or(false) {
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            __option_0 = Some(_p_val);
        }
        let _p_val = __option_0;
        let eventual1 = _p_val;
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let toggle2 = _p_val;
        _p_config.set_toggle("toggle2", _p_val);
        let mut __option_0: Option<Nested> = None;
        if _p_config.get_toggle("toggle2").unwrap_or(false) {
            let _p_val = binary_codec::variable::read_object(_p_bytes, None, _p_config)?;
            __option_0 = Some(_p_val);
        }
        let _p_val = __option_0;
        let eventual2 = _p_val;
        let _p_val = binary_codec::dynamics::read_small_dynamic_unsigned(
            _p_bytes,
            _p_config,
            3u8,
        )?;
        let length = _p_val;
        _p_config.set_length("len", _p_val as usize);
        let mut __val_0 = Vec::<bool>::with_capacity(3usize);
        for _ in 0..3usize {
            let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
            __val_0.push(_p_val);
        }
        let _p_val = TryInto::<[bool; 3usize]>::try_into(__val_0)
            .expect("Failed to convert Vec to array");
        let bools = _p_val;
        let _p_len = binary_codec::utils::get_read_size(
            _p_bytes,
            Some("len"),
            _p_config,
        )?;
        let mut __val_0 = Vec::<u8>::with_capacity(_p_len);
        for _ in 0.._p_len {
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            __val_0.push(_p_val);
        }
        let _p_val = __val_0;
        let array1 = _p_val;
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let boollie = _p_val;
        let _p_val = binary_codec::variable::read_string(
            _p_bytes,
            Some("len"),
            _p_config,
        )?;
        let str = _p_val;
        let mut _p_map = std::collections::HashMap::<u8, i32>::new();
        while _p_config.pos < _p_bytes.len() {
            let _p_key;
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            _p_key = _p_val;
            let _p_value;
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            _p_value = _p_val;
            _p_map.insert(_p_key, _p_value);
        }
        let _p_val = _p_map;
        let my_map = _p_val;
        Ok(Self {
            toggle,
            eventual1,
            toggle2,
            eventual2,
            length,
            bools,
            array1,
            boollie,
            str,
            my_map,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ExampleObject {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "toggle",
            "eventual1",
            "toggle2",
            "eventual2",
            "length",
            "bools",
            "array1",
            "boollie",
            "str",
            "my_map",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.toggle,
            &self.eventual1,
            &self.toggle2,
            &self.eventual2,
            &self.length,
            &self.bools,
            &self.array1,
            &self.boollie,
            &self.str,
            &&self.my_map,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "ExampleObject",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ExampleObject {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ExampleObject {
    #[inline]
    fn eq(&self, other: &ExampleObject) -> bool {
        self.toggle == other.toggle && self.toggle2 == other.toggle2
            && self.length == other.length && self.boollie == other.boollie
            && self.eventual1 == other.eventual1 && self.eventual2 == other.eventual2
            && self.bools == other.bools && self.array1 == other.array1
            && self.str == other.str && self.my_map == other.my_map
    }
}
enum Nested {
    A(u32),
    B(u64),
}
impl binary_codec::BinarySerializer for Nested {
    fn to_bytes(
        &self,
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<Vec<u8>, binary_codec::SerializationError> {
        let mut bytes = Vec::new();
        Self::write_bytes(self, &mut bytes, config)?;
        Ok(bytes)
    }
    fn write_bytes(
        &self,
        buffer: &mut Vec<u8>,
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<(), binary_codec::SerializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new();
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = buffer;
        match self {
            Self::A(f0) => {
                let _p_disc: u8 = 0u8;
                binary_codec::fixed_int::FixedInt::write(_p_disc, _p_bytes, _p_config)?;
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
            Self::B(f0) => {
                let _p_disc: u8 = 1u8;
                binary_codec::fixed_int::FixedInt::write(_p_disc, _p_bytes, _p_config)?;
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
        }
        Ok(())
    }
}
impl binary_codec::BinaryDeserializer for Nested {
    fn from_bytes(
        bytes: &[u8],
        config: Option<&mut binary_codec::SerializerConfig>,
    ) -> Result<Self, binary_codec::DeserializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new();
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = bytes;
        let _p_disc = _p_config
            .discriminator
            .take()
            .unwrap_or(binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?);
        match _p_disc {
            0u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let f0 = _p_val;
                Ok(Self::A(f0))
            }
            1u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let f0 = _p_val;
                Ok(Self::B(f0))
            }
            _ => Err(binary_codec::DeserializationError::UnknownDiscriminant(_p_disc)),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Nested {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Nested::A(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "A", &__self_0)
            }
            Nested::B(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "B", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Nested {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Nested {
    #[inline]
    fn eq(&self, other: &Nested) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (Nested::A(__self_0), Nested::A(__arg1_0)) => __self_0 == __arg1_0,
                (Nested::B(__self_0), Nested::B(__arg1_0)) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
