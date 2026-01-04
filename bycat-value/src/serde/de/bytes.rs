use crate::Bytes;
use core::marker::PhantomData;
use serde::{
    de::{self},
    forward_to_deserialize_any,
};

pub struct BytesVisitor;

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        formatter.write_str("Expected bytes")
    }

    fn visit_byte_buf<E>(self, v: alloc::vec::Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bytes::new(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bytes::from_slice(v))
    }
}

impl<'de> de::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for Bytes
where
    E: de::Error,
{
    type Deserializer = BytesDeserializer<Bytes, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        BytesDeserializer::new(self)
    }
}

pub struct BytesDeserializer<T, E>(T, PhantomData<E>);

impl<T, E> BytesDeserializer<T, E> {
    pub fn new(value: T) -> BytesDeserializer<T, E> {
        BytesDeserializer(value, PhantomData)
    }
}

impl<'de, T: AsRef<[u8]>, E> de::Deserializer<'de> for BytesDeserializer<T, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bytes(self.0.as_ref())
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map unit_struct option
        tuple_struct struct tuple ignored_any identifier newtype_struct enum
    }
}
