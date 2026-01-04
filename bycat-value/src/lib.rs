#![no_std]

extern crate alloc;

mod access;
mod bytes;
pub mod convert;
mod floating;
#[cfg(feature = "interner")]
mod interner;
mod list;
mod macros;
mod map;
mod merge;
mod number;
mod string;
mod time;
mod value;

#[cfg(feature = "rquickjs")]
mod rquickjs;

#[cfg(feature = "serde")]
pub mod serde;

pub use self::{
    bytes::Bytes,
    list::List,
    map::Map,
    merge::merge,
    number::Number,
    string::String,
    time::{Date, DateTime, Time, TimeZone},
    value::Value,
};

#[cfg(feature = "serde")]
pub use self::serde::{from_value, to_value};

#[cfg(feature = "interner")]
pub use self::interner::Interner;
