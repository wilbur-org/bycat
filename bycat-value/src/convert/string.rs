use alloc::string::ToString;

use crate::{String, Value};

impl<'a> From<&'a str> for String {
    fn from(value: &'a str) -> Self {
        String::new(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Value::String(value.into())
    }
}

impl<'a> From<alloc::string::String> for Value {
    fn from(value: alloc::string::String) -> Self {
        Value::String(value.into())
    }
}

impl TryFrom<Value> for String {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(str) => Ok(str),
            _ => Err("Value not a string"),
        }
    }
}
