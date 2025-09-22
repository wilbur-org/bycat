use crate::{Bytes, Date, DateTime, List, Map, Number, String, Time, Value};

macro_rules! is_method {
    ($check: ident, $ty: ident) => {
        pub fn $check(&self) -> bool {
            match self {
                Value::$ty(_) => true,
                _ => false,
            }
        }
    };
}

macro_rules! into_method {
    ($into: ident, $ty: ident, $oty: ty) => {
        pub fn $into(self) -> Result<$oty, Value> {
            match self {
                Value::$ty(v) => Ok(v),
                _ => Err(self),
            }
        }
    };
}

macro_rules! as_method {
    ($as: ident, $as_mut: ident, $ty: ident, $oty: ty) => {
        pub fn $as(&self) -> Option<&$oty> {
            match &self {
                Value::$ty(v) => Some(v),
                _ => None,
            }
        }

        pub fn $as_mut(&mut self) -> Option<&mut $oty> {
            match self {
                Value::$ty(v) => Some(v),
                _ => None,
            }
        }
    };
}

mod sealed {
    pub trait Sealed {}

    impl<'a> Sealed for &'a str {}

    impl<'a> Sealed for usize {}

    impl<'a> Sealed for i32 {}
}

pub trait Key: sealed::Sealed {
    fn get(self, value: &Value) -> Option<&Value>;
    fn get_mut(self, value: &mut Value) -> Option<&mut Value>;
}

impl<'a> Key for &'a str {
    fn get(self, value: &Value) -> Option<&Value> {
        match value {
            Value::Map(map) => map.get(self),
            _ => None,
        }
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        match value {
            Value::Map(map) => map.get_mut(self),
            _ => None,
        }
    }
}

impl Key for usize {
    fn get(self, value: &Value) -> Option<&Value> {
        match value {
            Value::List(list) => list.get(self),
            _ => None,
        }
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        match value {
            Value::List(list) => list.get_mut(self),
            _ => None,
        }
    }
}

impl Key for i32 {
    fn get(self, value: &Value) -> Option<&Value> {
        (self as usize).get(value)
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        (self as usize).get_mut(value)
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    is_method!(is_string, String);
    is_method!(is_bytes, Bytes);
    is_method!(is_bool, Bool);
    is_method!(is_list, List);
    is_method!(is_map, Map);
    is_method!(is_time, Time);
    is_method!(is_date, Date);
    is_method!(is_datetime, DateTime);

    as_method!(as_number, as_number_mut, Number, Number);
    as_method!(as_string, as_string_mut, String, String);
    as_method!(as_bytes, as_bytes_mut, Bytes, Bytes);
    as_method!(as_bool, as_bool_mut, Bool, bool);
    as_method!(as_list, as_list_mut, List, List);
    as_method!(as_map, as_map_mut, Map, Map<String, Value>);
    as_method!(as_time, as_time_mut, Time, Time);
    as_method!(as_datetime, as_datetime_mut, DateTime, DateTime);
    as_method!(as_date, as_date_mut, Date, Date);

    into_method!(into_string, String, String);
    into_method!(into_bytes, Bytes, Bytes);
    into_method!(into_bool, Bool, bool);
    into_method!(into_list, List, List);
    into_method!(into_map, Map, Map);
    into_method!(into_number, Number, Number);
    into_method!(into_time, Time, Time);
    into_method!(into_datetime, DateTime, DateTime);
    into_method!(into_date, Date, Date);

    pub fn get<T: Key>(&self, key: T) -> Option<&Value> {
        key.get(self)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        key.get_mut(self)
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        match self {
            Self::List(list) => list.push(value),
            _ => {}
        }
    }

    pub fn try_push<T>(&mut self, value: T) -> Result<(), T>
    where
        T: Into<Value>,
    {
        match self {
            Self::List(list) => {
                list.push(value);
                Ok(())
            }
            _ => Err(value),
        }
    }

    pub fn set(&mut self, key: &str, value: impl Into<Value>) -> Option<Value> {
        match self {
            Self::Map(map) => map.insert(key, value),
            _ => None,
        }
    }

    pub fn try_set<T>(&mut self, key: &str, value: T) -> Result<Option<Value>, T>
    where
        T: Into<Value>,
    {
        match self {
            Self::Map(map) => Ok(map.insert(key, value)),
            _ => Err(value),
        }
    }
}
