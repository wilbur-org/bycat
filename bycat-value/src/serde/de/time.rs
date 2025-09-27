use core::{marker::PhantomData, str::FromStr};

use alloc::string::ToString;
use serde::{de, forward_to_deserialize_any};

use crate::{
    Date, DateTime, Time,
    serde::de::{error::DeserializerError, string::StringDeserializer},
};

pub struct DateVisitor<T>(PhantomData<T>, &'static str);

impl<'de, T> de::Visitor<'de> for DateVisitor<T>
where
    T: FromStr,
{
    type Value = T;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        formatter.write_str(self.1)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(|_| de::Error::custom(self.1))
    }
}

impl<'de> de::Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor(PhantomData, "time"))
    }
}

impl<'de> de::Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor(PhantomData, "date"))
    }
}

impl<'de> de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor(PhantomData, "datetime"))
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for Date
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<Date, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringDeserializer::new(self)
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for Time
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<Time, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringDeserializer::new(self)
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for DateTime
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<DateTime, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringDeserializer::new(self)
    }
}
