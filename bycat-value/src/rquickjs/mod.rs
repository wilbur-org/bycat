use crate::time::TimeZone;
use crate::{Date, DateTime, String, Time};
pub use crate::{List, Map, Value};
use alloc::string::ToString;
use klaver_util::rquickjs::{
    self, Array, Ctx, FromJs, IntoJs, IteratorJs, Object, String as JsString, Type,
    Value as JsValue,
};
use klaver_util::rquickjs::{Atom, FromAtom, IntoAtom};

use klaver_util::{Date as JsDate, Iterable, IteratorIter, Map as JsMap, Pair, Set as JsSet};

macro_rules! un {
    ($expr: expr) => {
        $expr.map_err(|v| rquickjs::Error::new_from_js(v.type_name(), "value"))
    };
}

impl<'js> FromJs<'js> for Value {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        match value.type_of() {
            Type::Bool => Ok(Value::Bool(value.as_bool().unwrap())),
            Type::String => Ok(Value::String(
                un!(value.try_into_string())?.to_string()?.into(),
            )),
            Type::Int => Ok(Value::Number(value.as_int().unwrap().into())),
            Type::Float => Ok(Value::Number(value.as_float().unwrap().into())),
            Type::Null | Type::Undefined => Ok(Value::Null),
            Type::Array => {
                let array = un!(value.try_into_array())?;
                Ok(Value::List(
                    array.iter::<Value>().map(|m| m).collect::<Result<_, _>>()?,
                ))
            }
            Type::Object => {
                if JsDate::is(ctx, &value)? {
                    let date = JsDate::from_js(ctx, value)?;

                    let (year, month, day, h, m, s) = (
                        date.utc_year()?,
                        date.utc_month()?,
                        date.utc_date()?,
                        date.utc_hours()?,
                        date.utc_minutes()?,
                        date.utc_seconds()?,
                    );

                    let date = DateTime::new(
                        Date::new(year as _, month, day),
                        Time::from_hms(h as _, m as _, s as _),
                        TimeZone::UTC,
                    );

                    let ret = Ok(Value::DateTime(date));

                    return ret;
                } else if JsMap::is(ctx, &value)? {
                    let m = JsMap::from_js(ctx, value)?;
                    let mut map = Map::default();

                    let entries = IteratorIter::new(ctx.clone(), m.entries()?);

                    for next in entries {
                        let Pair(k, v) = next?.get::<Pair<String, Value>>()?;
                        map.insert(k, v);
                    }

                    Ok(Value::Map(map))
                } else if JsSet::is(ctx, &value)? {
                    let m = JsSet::from_js(ctx, value)?;
                    let mut list = List::default();

                    let entries = IteratorIter::new(ctx.clone(), m.entries()?);

                    for next in entries {
                        let Pair(_, v) = next?.get::<Pair<i32, Value>>()?;
                        list.push(v);
                    }

                    Ok(Value::List(list))
                } else {
                    let object = un!(value.try_into_object())?;

                    let mut map = Map::default();
                    for k in object.keys::<String>() {
                        let k = k?;
                        let v = object.get::<_, Value>(&k)?;
                        map.insert(k, v);
                    }

                    Ok(Value::Map(map))
                }
            }
            Type::Exception => {
                let exption = un!(value.try_into_exception())?;
                Ok(Value::String(exption.to_string().into()))
            }
            _ => Err(rquickjs::Error::new_from_js("value", "value")),
        }
    }
}

impl<'js> IntoJs<'js> for Value {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<JsValue<'js>> {
        let val = match self {
            Value::Bool(b) => JsValue::new_bool(ctx.clone(), b),
            Value::String(t) => JsString::from_str(ctx.clone(), t.as_str())?.into(),
            Value::Map(map) => map.into_js(ctx)?,
            Value::List(list) => list.into_js(ctx)?,
            Value::Bytes(bs) => rquickjs::ArrayBuffer::new(ctx.clone(), bs)?.into_value(),
            // Value::Date(_) => todo!(),
            Value::DateTime(datetime) => {
                let js_date = JsDate::from_str(ctx, &datetime.to_string())?;
                js_date.into_js(ctx)?
            }
            // Value::Time(_) => todo!(),
            Value::Number(n) => {
                if n.is_float() {
                    JsValue::new_float(ctx.clone(), n.as_f64())
                } else {
                    JsValue::new_int(ctx.clone(), n.as_i32())
                }
            }
            Value::Null => JsValue::new_null(ctx.clone()),
            _ => return Err(rquickjs::Error::new_into_js("value", "value")),
        };

        Ok(val)
    }
}

impl<'js> FromJs<'js> for String {
    fn from_js(_ctx: &Ctx<'js>, value: JsValue<'js>) -> rquickjs::Result<Self> {
        Ok(String::new(value.get()?))
    }
}

impl<'js> IntoJs<'js> for String {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<JsValue<'js>> {
        let ret = rquickjs::String::from_str(ctx.clone(), self.as_str())?;
        Ok(ret.into_value())
    }
}

impl<'js> FromAtom<'js> for String {
    fn from_atom(atom: rquickjs::Atom<'js>) -> rquickjs::Result<Self> {
        let string = atom.to_string()?;
        Ok(String::new(string))
    }
}

impl<'js> IntoAtom<'js> for String {
    fn into_atom(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Atom<'js>> {
        Atom::from_str(ctx.clone(), self.as_str())
    }
}

impl<'a, 'js> IntoAtom<'js> for &'a String {
    fn into_atom(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Atom<'js>> {
        Atom::from_str(ctx.clone(), self.as_str())
    }
}

impl<'js> FromJs<'js> for List {
    fn from_js(ctx: &Ctx<'js>, value: JsValue<'js>) -> rquickjs::Result<Self> {
        let iter = Iterable::from_js(ctx, value)?;
        let iter = IteratorIter::new(ctx.clone(), iter.iterator()?);
        let mut list = List::default();

        for next in iter {
            let next = next?;
            list.push(next.get::<Value>()?);
        }

        Ok(list)
    }
}

impl<'js> IntoJs<'js> for List {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<JsValue<'js>> {
        let items = self
            .into_iter()
            .map(|value| value.into_js(ctx))
            .collect_js::<Array>(ctx)?;

        Ok(items.into_value())
    }
}

impl<'js> FromJs<'js> for Map {
    fn from_js(ctx: &Ctx<'js>, value: JsValue<'js>) -> rquickjs::Result<Self> {
        if JsMap::is(ctx, &value)? {
            let m = value.get::<JsMap>()?;
            let mut map = Map::default();

            let entries = IteratorIter::new(ctx.clone(), m.entries()?);

            for next in entries {
                let Pair(k, v) = next?.get::<Pair<String, Value>>()?;
                map.insert(k, v);
            }
            Ok(map)
        } else {
            let obj = value.get::<Object>()?;
            let mut map = Map::default();
            for k in obj.keys::<String>() {
                let k = k?;
                let v = obj.get::<_, Value>(&k)?;
                map.insert(k, v);
            }
            Ok(map)
        }
    }
}

impl<'js> IntoJs<'js> for Map {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<JsValue<'js>> {
        let obj = rquickjs::Object::new(ctx.clone())?;
        for (k, v) in self {
            obj.set(k.as_str(), v.into_js(ctx)?)?;
        }
        Ok(obj.into_value())
    }
}
