use core::marker::PhantomData;

use serde::{
    de::{self, IntoDeserializer},
    forward_to_deserialize_any,
};

use crate::{String, Value, serde::de::error::unexpected};

impl<'de, E> de::IntoDeserializer<'de, E> for Value
where
    E: de::Error,
{
    type Deserializer = ValueDeserializer<E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer::new(self)
    }
}

pub struct ValueDeserializer<E> {
    value: Value,
    error: PhantomData<fn() -> E>,
}

impl<E> ValueDeserializer<E> {
    pub fn new(value: Value) -> Self {
        ValueDeserializer {
            value,
            error: Default::default(),
        }
    }

    pub fn into_value(self) -> Value {
        self.value
    }
}

impl<'de, E> de::Deserializer<'de> for ValueDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(n) => n.into_deserializer().deserialize_any(visitor),
            Value::String(v) => visitor.visit_str(&*v),
            Value::Null => visitor.visit_none(),
            Value::List(v) => visitor.visit_seq(de::value::SeqDeserializer::new(v.into_iter())),
            Value::Map(v) => visitor.visit_map(de::value::MapDeserializer::new(v.into_iter())),
            Value::Bytes(v) => visitor.visit_byte_buf(v.to_vec()),
            Value::Time(time) => time.into_deserializer().deserialize_any(visitor),
            Value::DateTime(date) => date.into_deserializer().deserialize_any(visitor),
            Value::Date(date) => date.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Null => self.deserialize_any(visitor),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let (variant, value) = match self.value {
            Value::Map(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(de::Error::invalid_type(
                    unexpected(&other),
                    &"string or map",
                ));
            }
        };

        let d = EnumDeserializer {
            variant,
            value,
            error: Default::default(),
        };
        visitor.visit_enum(d)
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map unit_struct
        tuple_struct struct tuple ignored_any identifier
    }
}

struct EnumDeserializer<E> {
    variant: String,
    value: Option<Value>,
    error: PhantomData<fn() -> E>,
}

impl<'de, E> de::EnumAccess<'de> for EnumDeserializer<E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = VariantDeserializer<Self::Error>;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, VariantDeserializer<Self::Error>), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let visitor = VariantDeserializer {
            value: self.value,
            error: Default::default(),
        };
        seed.deserialize(ValueDeserializer::new(Value::String(self.variant)))
            .map(|v| (v, visitor))
    }
}

struct VariantDeserializer<E> {
    value: Option<Value>,
    error: PhantomData<fn() -> E>,
}

impl<'de, E> de::VariantAccess<'de> for VariantDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(ValueDeserializer::new(value)),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(ValueDeserializer::new(value)),
            None => Err(de::Error::invalid_type(
                de::Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Value::List(v)) => de::Deserializer::deserialize_any(
                de::value::SeqDeserializer::new(v.into_iter()),
                visitor,
            ),
            Some(other) => Err(de::Error::invalid_type(
                unexpected(&other),
                &"tuple variant",
            )),
            None => Err(de::Error::invalid_type(
                de::Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Value::Map(v)) => de::Deserializer::deserialize_any(
                de::value::MapDeserializer::new(v.into_iter().map(|(k, v)| (Value::String(k), v))),
                visitor,
            ),
            Some(other) => Err(de::Error::invalid_type(
                unexpected(&other),
                &"struct variant",
            )),
            None => Err(de::Error::invalid_type(
                de::Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}
