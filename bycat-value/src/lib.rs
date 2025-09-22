#![no_std]

extern crate alloc;

mod access;
mod bytes;
pub mod convert;
mod floating;
mod list;
mod map;
mod number;
mod string;
mod time;
mod value;

#[cfg(feature = "serde")]
pub mod serde;

pub use self::{
    bytes::Bytes,
    list::List,
    map::Map,
    number::Number,
    string::String,
    time::{Date, DateTime, Time},
    value::Value,
};

#[cfg(feature = "serde")]
pub use self::serde::to_value;
