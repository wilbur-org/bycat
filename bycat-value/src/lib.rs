#![no_std]

extern crate alloc;

mod access;
mod bytes;
pub mod convert;
mod floating;
mod list;
mod map;
mod merge;
mod number;
mod string;
mod time;
mod value;

mod macros;

#[cfg(feature = "serde")]
pub mod serde;

pub use self::{
    bytes::Bytes,
    list::List,
    map::Map,
    merge::merge,
    number::Number,
    string::String,
    time::{Date, DateTime, Time},
    value::Value,
};

#[cfg(feature = "serde")]
pub use self::serde::{from_value, to_value};
