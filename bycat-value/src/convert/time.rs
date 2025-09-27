#[cfg(feature = "chrono")]
use chrono::Offset;
#[cfg(feature = "chrono")]
use chrono::{Datelike, Timelike};

use crate::Value;
#[cfg(feature = "chrono")]
use crate::time::TimeZone;
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
        use crate::time::TimeZone;

        DateTime::new(value.date().into(), value.time().into(), TimeZone::UTC)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for Value {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Value::DateTime(value.into())
    }
}

#[cfg(feature = "chrono")]
impl<T> From<chrono::DateTime<T>> for DateTime
where
    T: chrono::TimeZone,
{
    fn from(value: chrono::DateTime<T>) -> Self {
        let offset = value.offset().fix();
        let date = value.date_naive();
        let time = value.time();
        DateTime::new(date.into(), time.into(), offset.into())
    }
}

#[cfg(feature = "chrono")]
impl<T> From<chrono::DateTime<T>> for Value
where
    T: chrono::TimeZone,
{
    fn from(value: chrono::DateTime<T>) -> Self {
        Value::DateTime(value.into())
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::FixedOffset> for TimeZone {
    fn from(value: chrono::FixedOffset) -> Self {
        TimeZone::from_secs(value.local_minus_utc())
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<TimeZone> for chrono::FixedOffset {
    type Error = &'static str;
    fn try_from(value: TimeZone) -> Result<Self, Self::Error> {
        if value.offset() < 0 {
            chrono::FixedOffset::west_opt(value.offset())
        } else {
            chrono::FixedOffset::east_opt(value.offset())
        }
        .ok_or_else(|| "Invalid timezone")
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<DateTime> for chrono::DateTime<chrono::FixedOffset> {
    type Error = &'static str;

    fn try_from(value: DateTime) -> Result<Self, Self::Error> {
        let datetime: chrono::NaiveDateTime = value.try_into()?;
        let timezone: chrono::FixedOffset = value.time_zone().try_into()?;
        datetime
            .and_local_timezone(timezone)
            .single()
            .ok_or_else(|| "Invalid datetime")
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Date> for chrono::NaiveDate {
    type Error = &'static str;

    fn try_from(value: Date) -> Result<Self, Self::Error> {
        chrono::NaiveDate::from_ymd_opt(
            value.year() as i32,
            value.month() as u32,
            value.day() as u32,
        )
        .ok_or_else(|| "Invalid date")
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Time> for chrono::NaiveTime {
    type Error = &'static str;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        chrono::NaiveTime::from_hms_nano_opt(
            value.hour() as u32,
            value.minute() as u32,
            value.second() as u32,
            value.nanosecond() as u32,
        )
        .ok_or_else(|| "Invalid time")
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<DateTime> for chrono::NaiveDateTime {
    type Error = &'static str;

    fn try_from(value: DateTime) -> Result<Self, Self::Error> {
        let date = chrono::NaiveDate::try_from(value.date())?;
        let time = chrono::NaiveTime::try_from(value.time())?;
        Ok(chrono::NaiveDateTime::new(date, time))
    }
}
