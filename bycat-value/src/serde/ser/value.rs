use crate::Value;

impl serde::ser::Serialize for Value {
    fn serialize<S: serde::ser::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match *self {
            Value::Bool(v) => s.serialize_bool(v),
            Value::String(ref v) => s.serialize_str(v),
            Value::Number(n) => n.serialize(s),
            Value::Null => s.serialize_none(),
            Value::List(ref v) => v.serialize(s),
            Value::Map(ref v) => v.serialize(s),
            Value::Bytes(ref v) => s.serialize_bytes(v.as_ref()),
            // Value::Time(ref m) => m.serialize(s),
            // Value::Date(ref m) => m.serialize(s),
            // Value::DateTime(ref m) => m.serialize(s),
            _ => todo!(),
        }
    }
}
