mod bytes;
mod error;
mod list;
mod map;
mod number;
mod serializer;
mod string;
mod time;
mod value;

pub trait HasSerializer {
    type Serializer: serde::ser::Serializer<Ok = Self> + Default;
}

use crate::Value;

pub use self::error::SerializerError;

pub fn to_value<T: serde::ser::Serialize>(value: T) -> Result<Value, SerializerError> {
    value.serialize(self::serializer::Serializer)
}
