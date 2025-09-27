use crate::number::Number;
use core::fmt;
use core::marker::PhantomData;
use serde::{de, forward_to_deserialize_any};

use super::error::DeserializerError;

pub(crate) fn unexpected(value: &Number) -> serde::de::Unexpected {
    match *value {
        Number::U8(n) => serde::de::Unexpected::Unsigned(n as u64),
        Number::U16(n) => serde::de::Unexpected::Unsigned(n as u64),
        Number::U32(n) => serde::de::Unexpected::Unsigned(n as u64),
        Number::U64(n) => serde::de::Unexpected::Unsigned(n),
        Number::I8(n) => serde::de::Unexpected::Signed(n as i64),
        Number::I16(n) => serde::de::Unexpected::Signed(n as i64),
        Number::I32(n) => serde::de::Unexpected::Signed(n as i64),
        Number::I64(n) => serde::de::Unexpected::Signed(n),
        Number::F32(n) => serde::de::Unexpected::Float(n as f64),
        Number::F64(n) => serde::de::Unexpected::Float(n),
    }
}

pub struct NumberVisitor;

impl<'de> de::Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("any numeric value")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_i16<E>(self, value: i16) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_i32<E>(self, value: i32) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_u8<E>(self, value: u8) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_u16<E>(self, value: u16) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_u32<E>(self, value: u32) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_f32<E>(self, value: f32) -> Result<Number, E> {
        Ok(value.into())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Number, E> {
        Ok(value.into())
    }
}

pub struct NumberDeserializer<E> {
    value: Number,
    error: PhantomData<fn() -> E>,
}

impl<E> NumberDeserializer<E> {
    pub fn new(value: Number) -> Self {
        NumberDeserializer {
            value,
            error: Default::default(),
        }
    }

    pub fn into_number(self) -> Number {
        self.value
    }
}

impl<'de, E> de::Deserializer<'de> for NumberDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Number::U8(v) => visitor.visit_u8(v),
            Number::U16(v) => visitor.visit_u16(v),
            Number::U32(v) => visitor.visit_u32(v),
            Number::U64(v) => visitor.visit_u64(v),
            Number::I8(v) => visitor.visit_i8(v),
            Number::I16(v) => visitor.visit_i16(v),
            Number::I32(v) => visitor.visit_i32(v),
            Number::I64(v) => visitor.visit_i64(v),
            Number::F32(v) => visitor.visit_f32(v),
            Number::F64(v) => visitor.visit_f64(v),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map unit_struct option
        tuple_struct struct tuple ignored_any identifier newtype_struct enum
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for NumberDeserializer<E>
where
    E: de::Error,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for Number
where
    E: de::Error,
{
    type Deserializer = NumberDeserializer<E>;

    fn into_deserializer(self) -> Self::Deserializer {
        NumberDeserializer::new(self)
    }
}

impl<'de> de::Deserializer<'de> for Number {
    type Error = DeserializerError;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        NumberDeserializer::new(self).deserialize_any(visitor)
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        NumberDeserializer::new(self).deserialize_option(visitor)
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        NumberDeserializer::new(self).deserialize_enum(name, variants, visitor)
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        NumberDeserializer::new(self).deserialize_newtype_struct(name, visitor)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map unit_struct
        tuple_struct struct tuple ignored_any identifier
    }
}

impl<'de> de::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}
