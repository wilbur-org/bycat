use alloc::vec::Vec;

use crate::map::Map;

use crate::{List, String, Value};

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Null
    }
}

impl TryFrom<Value> for bool {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err("Value not a boolean"),
        }
    }
}

impl From<Map<String, Value>> for Value {
    fn from(value: Map<String, Value>) -> Self {
        Value::Map(value)
    }
}

impl From<List> for Value {
    fn from(value: List) -> Self {
        Value::List(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(value.into())
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Value {
    fn from(value: uuid::Uuid) -> Self {
        use alloc::string::ToString;
        Value::String(value.hyphenated().to_string().into())
    }
}

#[cfg(feature = "uuid")]
impl TryFrom<Value> for uuid::Uuid {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(str) => {
                use core::str::FromStr;
                uuid::Uuid::from_str(&str).map_err(|_| "Invalid uuid")
            }
            Value::Bytes(bs) => uuid::Uuid::from_slice(&*bs).map_err(|_| "Invalid uuid"),
            _ => Err("Invalid uuid"),
        }
    }
}
