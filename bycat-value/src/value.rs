use core::fmt;

use crate::access::Key;
use crate::bytes::Bytes;

use crate::number::Number;
use crate::time::{Date, DateTime, Time};
use crate::{list::List, map::Map, string::String};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Bool(bool),
    String(String),
    Bytes(Bytes),
    List(List),
    Map(Map),
    Number(Number),
    DateTime(DateTime),
    Date(Date),
    Time(Time),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => v.fmt(f),
            Value::String(v) => v.fmt(f),
            Value::Map(v) => v.fmt(f),
            Value::List(v) => v.fmt(f),
            Value::Bytes(v) => write!(f, "<Bytes {}>", v.len()),
            Value::Date(v) => v.fmt(f),
            Value::DateTime(v) => v.fmt(f),
            Value::Time(v) => v.fmt(f),
            Value::Number(v) => v.fmt(f),
            Value::Null => write!(f, "null"),
        }
    }
}

impl<K: Key> core::ops::Index<K> for Value {
    type Output = Value;
    fn index(&self, index: K) -> &Self::Output {
        static NULL: Value = Value::Null;
        self.get(index).unwrap_or(&NULL)
    }
}
