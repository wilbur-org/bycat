use core::marker::PhantomData;

use alloc::string::ToString;
use serde::{
    de::{self, value},
    forward_to_deserialize_any,
};

use crate::String;

pub struct StringVisitor;

impl<'de> de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        formatter.write_str("Expected a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(String::from(v))
    }

    fn visit_string<E>(self, v: alloc::string::String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(String::from(v))
    }
}

impl<'de> de::Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor)
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for String
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<String, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringDeserializer::new(self)
    }
}

pub struct StringDeserializer<T, E>(T, PhantomData<E>);

impl<T, E> StringDeserializer<T, E> {
    pub fn new(value: T) -> StringDeserializer<T, E> {
        StringDeserializer(value, PhantomData)
    }
}

impl<'de, T: ToString, E> de::Deserializer<'de> for StringDeserializer<T, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_string(self.0.to_string())
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map unit_struct option
        tuple_struct struct tuple ignored_any identifier newtype_struct enum
    }
}
