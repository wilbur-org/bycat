use super::number;
use crate::{Map, String, value::Value};
use alloc::{
    borrow::ToOwned,
    string::{String as StdString, ToString},
    vec::Vec,
};
use core::{fmt, marker::PhantomData};
use serde::{de, forward_to_deserialize_any};

#[derive(Debug)]
pub enum Unexpected {
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Char(char),
    Str(StdString),
    Bytes(Vec<u8>),
    Unit,
    Option,
    NewtypeStruct,
    Seq,
    Map,
    Enum,
    UnitVariant,
    NewtypeVariant,
    TupleVariant,
    StructVariant,
    Other(StdString),
}

pub(crate) fn unexpected(value: &Value) -> serde::de::Unexpected {
    match *value {
        Value::Bool(b) => serde::de::Unexpected::Bool(b),
        Value::Number(ref n) => number::unexpected(n),
        Value::String(ref s) => serde::de::Unexpected::Str(s),
        Value::Null => serde::de::Unexpected::Option,
        Value::List(_) => serde::de::Unexpected::Seq,
        Value::Map(_) => serde::de::Unexpected::Map,
        Value::Bytes(ref b) => serde::de::Unexpected::Bytes(b),
        #[allow(unreachable_patterns)]
        _ => serde::de::Unexpected::Map,
    }
}

impl<'a> From<de::Unexpected<'a>> for Unexpected {
    fn from(unexp: de::Unexpected) -> Unexpected {
        match unexp {
            de::Unexpected::Bool(v) => Unexpected::Bool(v),
            de::Unexpected::Unsigned(v) => Unexpected::Unsigned(v),
            de::Unexpected::Signed(v) => Unexpected::Signed(v),
            de::Unexpected::Float(v) => Unexpected::Float(v),
            de::Unexpected::Char(v) => Unexpected::Char(v),
            de::Unexpected::Str(v) => Unexpected::Str(v.to_owned()),
            de::Unexpected::Bytes(v) => Unexpected::Bytes(v.to_owned()),
            de::Unexpected::Unit => Unexpected::Unit,
            de::Unexpected::Option => Unexpected::Option,
            de::Unexpected::NewtypeStruct => Unexpected::NewtypeStruct,
            de::Unexpected::Seq => Unexpected::Seq,
            de::Unexpected::Map => Unexpected::Map,
            de::Unexpected::Enum => Unexpected::Enum,
            de::Unexpected::UnitVariant => Unexpected::UnitVariant,
            de::Unexpected::NewtypeVariant => Unexpected::NewtypeVariant,
            de::Unexpected::TupleVariant => Unexpected::TupleVariant,
            de::Unexpected::StructVariant => Unexpected::StructVariant,
            de::Unexpected::Other(v) => Unexpected::Other(v.to_owned()),
        }
    }
}

impl Unexpected {
    pub fn to_unexpected(&self) -> de::Unexpected<'_> {
        match *self {
            Unexpected::Bool(v) => de::Unexpected::Bool(v),
            Unexpected::Unsigned(v) => de::Unexpected::Unsigned(v),
            Unexpected::Signed(v) => de::Unexpected::Signed(v),
            Unexpected::Float(v) => de::Unexpected::Float(v),
            Unexpected::Char(v) => de::Unexpected::Char(v),
            Unexpected::Str(ref v) => de::Unexpected::Str(v),
            Unexpected::Bytes(ref v) => de::Unexpected::Bytes(v),
            Unexpected::Unit => de::Unexpected::Unit,
            Unexpected::Option => de::Unexpected::Option,
            Unexpected::NewtypeStruct => de::Unexpected::NewtypeStruct,
            Unexpected::Seq => de::Unexpected::Seq,
            Unexpected::Map => de::Unexpected::Map,
            Unexpected::Enum => de::Unexpected::Enum,
            Unexpected::UnitVariant => de::Unexpected::UnitVariant,
            Unexpected::NewtypeVariant => de::Unexpected::NewtypeVariant,
            Unexpected::TupleVariant => de::Unexpected::TupleVariant,
            Unexpected::StructVariant => de::Unexpected::StructVariant,
            Unexpected::Other(ref v) => de::Unexpected::Other(v),
        }
    }
}

#[derive(Debug)]
pub enum DeserializerError {
    Custom(StdString),
    InvalidType(Unexpected, StdString),
    InvalidValue(Unexpected, StdString),
    InvalidLength(usize, StdString),
    UnknownVariant(StdString, &'static [&'static str]),
    UnknownField(StdString, &'static [&'static str]),
    MissingField(&'static str),
    DuplicateField(&'static str),
}

pub fn from_value<T: de::DeserializeOwned>(value: Value) -> Result<T, DeserializerError> {
    T::deserialize(value)
}

impl de::Error for DeserializerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializerError::Custom(msg.to_string())
    }

    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        DeserializerError::InvalidType(unexp.into(), exp.to_string())
    }

    fn invalid_value(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        DeserializerError::InvalidValue(unexp.into(), exp.to_string())
    }

    fn invalid_length(len: usize, exp: &dyn de::Expected) -> Self {
        DeserializerError::InvalidLength(len, exp.to_string())
    }

    fn unknown_variant(field: &str, expected: &'static [&'static str]) -> Self {
        DeserializerError::UnknownVariant(field.into(), expected)
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        DeserializerError::UnknownField(field.into(), expected)
    }

    fn missing_field(field: &'static str) -> Self {
        DeserializerError::MissingField(field)
    }

    fn duplicate_field(field: &'static str) -> Self {
        DeserializerError::DuplicateField(field)
    }
}

impl DeserializerError {
    pub fn to_error<E: de::Error>(&self) -> E {
        match *self {
            DeserializerError::Custom(ref msg) => E::custom(msg.clone()),
            DeserializerError::InvalidType(ref unexp, ref exp) => {
                E::invalid_type(unexp.to_unexpected(), &&**exp)
            }
            DeserializerError::InvalidValue(ref unexp, ref exp) => {
                E::invalid_value(unexp.to_unexpected(), &&**exp)
            }
            DeserializerError::InvalidLength(len, ref exp) => E::invalid_length(len, &&**exp),
            DeserializerError::UnknownVariant(ref field, exp) => E::unknown_variant(field, exp),
            DeserializerError::UnknownField(ref field, exp) => E::unknown_field(field, exp),
            DeserializerError::MissingField(field) => E::missing_field(field),
            DeserializerError::DuplicateField(field) => E::missing_field(field),
        }
    }

    pub fn into_error<E: de::Error>(self) -> E {
        self.to_error()
    }
}

impl de::StdError for DeserializerError {}

impl fmt::Display for DeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializerError::Custom(ref msg) => write!(f, "{}", msg),
            DeserializerError::InvalidType(ref unexp, ref exp) => {
                write!(
                    f,
                    "Invalid type {}. Expected {}",
                    unexp.to_unexpected(),
                    exp
                )
            }
            DeserializerError::InvalidValue(ref unexp, ref exp) => {
                write!(
                    f,
                    "Invalid value {}. Expected {}",
                    unexp.to_unexpected(),
                    exp
                )
            }
            DeserializerError::InvalidLength(len, ref exp) => {
                write!(f, "Invalid length {}. Expected {}", len, exp)
            }
            DeserializerError::UnknownVariant(ref field, exp) => {
                write!(
                    f,
                    "Unknown variant {}. Expected one of {}",
                    field,
                    exp.join(", ")
                )
            }
            DeserializerError::UnknownField(ref field, exp) => {
                write!(
                    f,
                    "Unknown field {}. Expected one of {}",
                    field,
                    exp.join(", ")
                )
            }
            DeserializerError::MissingField(field) => write!(f, "Missing field {}", field),
            DeserializerError::DuplicateField(field) => write!(f, "Duplicate field {}", field),
        }
    }
}

impl From<de::value::Error> for DeserializerError {
    fn from(e: de::value::Error) -> DeserializerError {
        DeserializerError::Custom(e.to_string())
    }
}
