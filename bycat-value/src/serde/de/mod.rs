use serde::de::IntoDeserializer;

use crate::Value;

mod bytes;
mod deserializer;
mod error;
mod list;
mod map;
mod number;
mod string;
mod time;
mod value;

pub use self::error::DeserializerError;

pub fn from_value<'de, T: serde::de::Deserialize<'de>>(
    value: Value,
) -> Result<T, DeserializerError> {
    T::deserialize(value.into_deserializer())
}
