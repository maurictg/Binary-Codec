#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::collections::HashMap;
use binary_codec_derive::{FromBytes, ToBytes};
mod out {}
struct Testt {
    #[toggles("1")]
    has1: bool,
    #[toggles("2")]
    has2: bool,
    #[toggles("3")]
    has3: bool,
    #[toggles("bool")]
    boolean_is_true: bool,
    #[multi_enum]
    multi: Vec<MultiEnum>,
    #[multi_enum]
    boolean: Boolean,
}
impl<T: Clone> binary_codec::BinarySerializer<T> for Testt {
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
        let _p_val = &self.has1;
        _p_config.set_toggle("1", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.has2;
        _p_config.set_toggle("2", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.has3;
        _p_config.set_toggle("3", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.boolean_is_true;
        _p_config.set_toggle("bool", *_p_val);
        binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?;
        let _p_val = &self.multi;
        for _p_val in _p_val {
            _p_config.discriminator = _p_config
                .get_next_multi_disc("multi", "MultiEnum");
            binary_codec::variable::write_object(_p_val, None, _p_bytes, _p_config)?;
        }
        let _p_val = &self.boolean;
        _p_config.discriminator = _p_config.get_next_multi_disc("boolean", "Boolean");
        binary_codec::variable::write_object(_p_val, None, _p_bytes, _p_config)?;
        Ok(())
    }
}
impl<T: Clone> binary_codec::BinaryDeserializer<T> for Testt {
    fn deserialize(
        bytes: &[u8],
        config: Option<&mut binary_codec::SerializerConfig<T>>,
    ) -> Result<Self, binary_codec::DeserializationError> {
        let mut _new_config = binary_codec::SerializerConfig::new(None);
        let _p_config = config.unwrap_or(&mut _new_config);
        let _p_bytes = bytes;
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let has1 = _p_val;
        _p_config.set_toggle("1", _p_val);
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let has2 = _p_val;
        _p_config.set_toggle("2", _p_val);
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let has3 = _p_val;
        _p_config.set_toggle("3", _p_val);
        let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?;
        let boolean_is_true = _p_val;
        _p_config.set_toggle("bool", _p_val);
        MultiEnum::configure_multi_disc(_p_config);
        let _p_len = _p_config.get_multi_disc_size("MultiEnum");
        let mut __val_0 = Vec::<MultiEnum>::with_capacity(_p_len);
        for _ in 0.._p_len {
            _p_config.discriminator = _p_config
                .get_next_multi_disc("multi", "MultiEnum");
            let _p_val = binary_codec::variable::read_object(_p_bytes, None, _p_config)?;
            __val_0.push(_p_val);
        }
        let _p_val = __val_0;
        let multi = _p_val;
        _p_config.discriminator = _p_config.get_next_multi_disc("boolean", "Boolean");
        let _p_val = binary_codec::variable::read_object(_p_bytes, None, _p_config)?;
        let boolean = _p_val;
        Ok(Self {
            has1,
            has2,
            has3,
            boolean_is_true,
            multi,
            boolean,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Testt {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "has1",
            "has2",
            "has3",
            "boolean_is_true",
            "multi",
            "boolean",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.has1,
            &self.has2,
            &self.has3,
            &self.boolean_is_true,
            &self.multi,
            &&self.boolean,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "Testt", names, values)
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Testt {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Testt {
    #[inline]
    fn eq(&self, other: &Testt) -> bool {
        self.has1 == other.has1 && self.has2 == other.has2 && self.has3 == other.has3
            && self.boolean_is_true == other.boolean_is_true && self.multi == other.multi
            && self.boolean == other.boolean
    }
}
#[no_discriminator]
enum MultiEnum {
    #[toggled_by = "1"]
    Value1(u8),
    #[toggled_by = "2"]
    Value2(u16),
    #[toggled_by = "3"]
    Value3(u32),
}
impl<T: Clone> binary_codec::BinarySerializer<T> for MultiEnum {
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
        _p_config.configure_multi_disc("MultiEnum", 0u8, "1");
        _p_config.configure_multi_disc("MultiEnum", 1u8, "2");
        _p_config.configure_multi_disc("MultiEnum", 2u8, "3");
        let _p_bytes = buffer;
        match self {
            Self::Value1(f0) => {
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
            Self::Value2(f0) => {
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
            Self::Value3(f0) => {
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
        }
        Ok(())
    }
}
impl MultiEnum {
    pub fn get_discriminator(&self) -> u8 {
        match self {
            Self::Value1(..) => 0u8,
            Self::Value2(..) => 1u8,
            Self::Value3(..) => 2u8,
        }
    }
}
impl MultiEnum {
    pub fn configure_multi_disc<T: Clone>(
        config: &mut binary_codec::SerializerConfig<T>,
    ) {
        let _p_config = config;
        _p_config.configure_multi_disc("MultiEnum", 0u8, "1");
        _p_config.configure_multi_disc("MultiEnum", 1u8, "2");
        _p_config.configure_multi_disc("MultiEnum", 2u8, "3");
    }
}
impl<T: Clone> binary_codec::BinaryDeserializer<T> for MultiEnum {
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
                Ok(Self::Value1(f0))
            }
            1u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let f0 = _p_val;
                Ok(Self::Value2(f0))
            }
            2u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let f0 = _p_val;
                Ok(Self::Value3(f0))
            }
            _ => Err(binary_codec::DeserializationError::UnknownDiscriminant(_p_disc)),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for MultiEnum {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            MultiEnum::Value1(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Value1", &__self_0)
            }
            MultiEnum::Value2(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Value2", &__self_0)
            }
            MultiEnum::Value3(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Value3", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for MultiEnum {}
#[automatically_derived]
impl ::core::cmp::PartialEq for MultiEnum {
    #[inline]
    fn eq(&self, other: &MultiEnum) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (MultiEnum::Value1(__self_0), MultiEnum::Value1(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                (MultiEnum::Value2(__self_0), MultiEnum::Value2(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                (MultiEnum::Value3(__self_0), MultiEnum::Value3(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[no_discriminator]
enum Boolean {
    #[toggled_by = "bool"]
    False(u8),
    #[toggled_by = "!bool"]
    True(u16),
}
impl<T: Clone> binary_codec::BinarySerializer<T> for Boolean {
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
        _p_config.configure_multi_disc("Boolean", 0u8, "bool");
        _p_config.configure_multi_disc("Boolean", 1u8, "!bool");
        let _p_bytes = buffer;
        match self {
            Self::False(f0) => {
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
            Self::True(f0) => {
                let _p_val = f0;
                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
            }
        }
        Ok(())
    }
}
impl Boolean {
    pub fn get_discriminator(&self) -> u8 {
        match self {
            Self::False(..) => 0u8,
            Self::True(..) => 1u8,
        }
    }
}
impl Boolean {
    pub fn configure_multi_disc<T: Clone>(
        config: &mut binary_codec::SerializerConfig<T>,
    ) {
        let _p_config = config;
        _p_config.configure_multi_disc("Boolean", 0u8, "bool");
        _p_config.configure_multi_disc("Boolean", 1u8, "!bool");
    }
}
impl<T: Clone> binary_codec::BinaryDeserializer<T> for Boolean {
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
                Ok(Self::False(f0))
            }
            1u8 => {
                let _p_val = binary_codec::fixed_int::FixedInt::read(
                    _p_bytes,
                    _p_config,
                )?;
                let f0 = _p_val;
                Ok(Self::True(f0))
            }
            _ => Err(binary_codec::DeserializationError::UnknownDiscriminant(_p_disc)),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Boolean {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Boolean::False(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "False", &__self_0)
            }
            Boolean::True(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "True", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Boolean {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Boolean {
    #[inline]
    fn eq(&self, other: &Boolean) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (Boolean::False(__self_0), Boolean::False(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                (Boolean::True(__self_0), Boolean::True(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
