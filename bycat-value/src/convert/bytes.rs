use alloc::vec::Vec;

use crate::{Bytes, Value};

impl From<Bytes> for Value {
    fn from(value: Bytes) -> Self {
        Value::Bytes(value)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Bytes::new(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(value.into())
    }
}
