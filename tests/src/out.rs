#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::collections::HashMap;
use binary_codec_derive::{FromBytes, ToBytes};
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
impl<T: Clone> binary_codec::BinarySerializer<T> for ExampleObject {
    fn serialize(
        &self,
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<Vec<u8>, binary_codec::SerializationError> {
        let mut bytes = Vec::new();
        Self::write_bytes(self, &mut bytes, config)?;
        Ok(bytes)
    }
    fn write_bytes(
        &self,
        buffer: &mut Vec<u8>,
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<(), binary_codec::SerializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new(None);
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
        let _p_val = &self.dyn_len_arr;
        let _p_len = _p_val.len();
        binary_codec::utils::write_size(_p_len, Some("__dynamic"), _p_bytes, _p_config)?;
        for _p_val in _p_val {
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
        }
        let _p_val = &self.my_map;
        let _p_len = _p_val.len();
        binary_codec::utils::write_size(_p_len, Some("__dynamic"), _p_bytes, _p_config)?;
        for (key, value) in _p_val {
            let _p_val = key;
            binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            let _p_val = value;
            binary_codec::variable::write_string(
                _p_val,
                Some("__dynamic"),
                _p_bytes,
                _p_config,
            )?;
        }
        let _p_val = &self.dyn_int;
        let _p_dyn = *_p_val as u128;
        binary_codec::dyn_int::write_dynint(_p_dyn, _p_bytes, _p_config)?;
        Ok(())
    }
}
impl<T: Clone> binary_codec::BinaryDeserializer<T> for ExampleObject {
    fn deserialize(
        bytes: &[u8],
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<Self, binary_codec::DeserializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new(None);
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
        let _p_len = binary_codec::utils::get_read_size(
            _p_bytes,
            Some("__dynamic"),
            _p_config,
        )?;
        let mut __val_0 = Vec::<u16>::with_capacity(_p_len);
        for _ in 0.._p_len {
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            __val_0.push(_p_val);
        }
        let _p_val = __val_0;
        let dyn_len_arr = _p_val;
        let _p_len = binary_codec::utils::get_read_size(
            _p_bytes,
            Some("__dynamic"),
            _p_config,
        )?;
        let mut _p_map = std::collections::HashMap::<u8, String>::with_capacity(_p_len);
        for _ in 0.._p_len {
            let _p_key;
            let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
            _p_key = _p_val;
            let _p_value;
            let _p_val = binary_codec::variable::read_string(
                _p_bytes,
                Some("__dynamic"),
                _p_config,
            )?;
            _p_value = _p_val;
            _p_map.insert(_p_key, _p_value);
        }
        let _p_val = _p_map;
        let my_map = _p_val;
        let _p_dyn = binary_codec::dyn_int::read_dynint(_p_bytes, _p_config)?;
        let _p_val = _p_dyn as u64;
        let dyn_int = _p_val;
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
            dyn_len_arr,
            my_map,
            dyn_int,
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
            "dyn_len_arr",
            "my_map",
            "dyn_int",
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
            &self.dyn_len_arr,
            &self.my_map,
            &&self.dyn_int,
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
            && self.dyn_int == other.dyn_int && self.eventual1 == other.eventual1
            && self.eventual2 == other.eventual2 && self.bools == other.bools
            && self.array1 == other.array1 && self.str == other.str
            && self.dyn_len_arr == other.dyn_len_arr && self.my_map == other.my_map
    }
}
enum Nested {
    A(u32),
    B(u64),
    C,
    D { x: u32 },
}
impl<T: Clone> binary_codec::BinarySerializer<T> for Nested {
    fn serialize(
        &self,
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<Vec<u8>, binary_codec::SerializationError> {
        let mut bytes = Vec::new();
        Self::write_bytes(self, &mut bytes, config)?;
        Ok(bytes)
    }
    fn write_bytes(
        &self,
        buffer: &mut Vec<u8>,
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<(), binary_codec::SerializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new(None);
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
            Self::C => {
                let _p_disc: u8 = 2u8;
                binary_codec::fixed_int::FixedInt::write(_p_disc, _p_bytes, _p_config)?;
            }
            Self::D { x } => {
                let _p_disc: u8 = 3u8;
                binary_codec::fixed_int::FixedInt::write(_p_disc, _p_bytes, _p_config)?;
                let _p_val = x;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
        }
        Ok(())
    }
}
impl Nested {
    pub fn get_discriminator(&self) -> u8 {
        match self {
            Self::A(..) => 0u8,
            Self::B(..) => 1u8,
            Self::C => 2u8,
            Self::D { .. } => 3u8,
        }
    }
}
impl<T: Clone> binary_codec::BinaryDeserializer<T> for Nested {
    fn deserialize(
        bytes: &[u8],
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<Self, binary_codec::DeserializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new(None);
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = bytes;
        let _p_disc = if let Some(disc) = _p_config.discriminator.take() {
            disc
        } else {
            binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?
        };
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
            2u8 => Ok(Self::C),
            3u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let x = _p_val;
                Ok(Self::D { x })
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
            Nested::C => ::core::fmt::Formatter::write_str(f, "C"),
            Nested::D { x: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "D",
                    "x",
                    &__self_0,
                )
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
                (Nested::D { x: __self_0 }, Nested::D { x: __arg1_0 }) => {
                    __self_0 == __arg1_0
                }
                _ => true,
            }
    }
}
