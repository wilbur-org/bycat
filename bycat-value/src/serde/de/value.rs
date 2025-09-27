use alloc::{collections::btree_map::BTreeMap, string::ToString, vec::Vec};

use crate::Value;

use core::fmt;
use serde::de;

pub struct ValueVisitor;

impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("any value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i8<E>(self, value: i8) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u8<E>(self, value: u8) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_char<E>(self, value: char) -> Result<Value, E> {
        Ok(Value::String(value.to_string().into()))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }

    // #[cfg(feature = "std")]
    fn visit_string<E>(self, value: alloc::string::String) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D: de::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(ValueVisitor)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<V: de::SeqAccess<'de>>(self, mut visitor: V) -> Result<Value, V::Error> {
        let mut values = Vec::new();
        while let Some(elem) = visitor.next_element::<Value>()? {
            values.push(elem);
        }
        Ok(Value::List(values.into()))
    }

    fn visit_map<V: de::MapAccess<'de>>(self, mut visitor: V) -> Result<Value, V::Error> {
        let mut values = BTreeMap::default();
        while let Some((key, value)) = visitor.next_entry()? {
            values.insert(key, value);
        }
        Ok(Value::Map(values.into()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Value, E> {
        Ok(Value::Bytes(v.to_vec().into()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Value, E> {
        Ok(Value::Bytes(v.into()))
    }

    // fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    // where
    //     E: de::Error,
    // {
    //     let mut buf = [0u8; 58];
    //     let mut writer = crate::format::Buf::new(&mut buf);
    //     alloc::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as i128", v)).unwrap();
    //     Err(de::Error::invalid_type(
    //         de::Unexpected::Other(writer.as_str()),
    //         &self,
    //     ))
    // }

    // fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    // where
    //     E: de::Error,
    // {
    //     let mut buf = [0u8; 57];
    //     let mut writer = crate::format::Buf::new(&mut buf);
    //     alloc::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as u128", v)).unwrap();
    //     Err(de::Error::invalid_type(
    //         de::Unexpected::Other(writer.as_str()),
    //         &self,
    //     ))
    // }

    // fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    // where
    //     A: de::EnumAccess<'de>,
    // {
    //     let _ = data;
    //     Err(de::Error::invalid_type(de::Unexpected::Enum, &self))
    // }
}

impl<'de> de::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}
