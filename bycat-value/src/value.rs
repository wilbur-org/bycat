use crate::bytes::Bytes;

use crate::number::Number;
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    date: Date,
    time: Time,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub date: u8,
    pub month: u8,
    pub year: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub secs: u32,
    pub frac: u32,
}
