#[cfg(feature = "chrono")]
use chrono::{Datelike, Timelike};

use crate::Value;
use crate::{Date, DateTime, Time};

impl From<Date> for Value {
    fn from(date: Date) -> Self {
        Value::Date(date)
    }
}

impl From<Time> for Value {
    fn from(time: Time) -> Self {
        Value::Time(time)
    }
}

impl From<DateTime> for Value {
    fn from(datetime: DateTime) -> Self {
        Value::DateTime(datetime)
    }
}

impl TryFrom<Value> for Date {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Date(date) = value {
            Ok(date)
        } else {
            Err("Value is not a Date")
        }
    }
}

impl TryFrom<Value> for Time {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Time(time) = value {
            Ok(time)
        } else {
            Err("Value is not a Time")
        }
    }
}

impl TryFrom<Value> for DateTime {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::DateTime(datetime) = value {
            Ok(datetime)
        } else {
            Err("Value is not a DateTime")
        }
    }
}

// Chrono

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for Date {
    fn from(value: chrono::NaiveDate) -> Self {
        Date::new(value.day() as _, value.month() as _, value.year() as _)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for Value {
    fn from(value: chrono::NaiveDate) -> Self {
        Value::Date(value.into())
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveTime> for Time {
    fn from(value: chrono::NaiveTime) -> Self {
        Time::from_hmsn(
            value.hour(),
            value.minute(),
            value.second(),
            value.nanosecond(),
        )
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveTime> for Value {
    fn from(value: chrono::NaiveTime) -> Self {
        Value::Time(value.into())
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for DateTime {
    fn from(value: chrono::NaiveDateTime) -> Self {
        DateTime::new(value.date().into(), value.time().into())
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for Value {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Value::DateTime(value.into())
    }
}

// #[cfg(feature = "chrono")]
// impl TryFrom<Date> for chrono::NaiveDate {
//     type Error = chrono::ParseError;

//     fn try_from(value: Date) -> Result<Self, Self::Error> {
//         chrono::NaiveDate::from_ymd_opt(
//             value.year() as i32,
//             value.month() as u32,
//             value.day() as u32,
//         )
//         .ok_or_else(|| chrono::ParseError::)
//     }
// }

// impl TryFrom<Time> for chrono::NaiveTime {
//     type Error = chrono::ParseError;

//     fn try_from(value: Time) -> Result<Self, Self::Error> {
//         chrono::NaiveTime::from_hms_nano_opt(
//             value.hour() as u32,
//             value.minute() as u32,
//             value.second() as u32,
//             value.nanosecond() as u32,
//         )
//         .ok_or_else(|| chrono::ParseError::Impossible)
//     }
// }

// impl TryFrom<DateTime> for chrono::NaiveDateTime {
//     type Error = chrono::ParseError;

//     fn try_from(value: DateTime) -> Result<Self, Self::Error> {
//         let date = chrono::NaiveDate::try_from(value.date())?;
//         let time = chrono::NaiveTime::try_from(value.time())?;
//         Ok(chrono::NaiveDateTime::new(date, time))
//     }
// }
