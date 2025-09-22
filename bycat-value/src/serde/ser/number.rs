use crate::Number;

impl serde::ser::Serialize for Number {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Number::U8(v) => s.serialize_u8(v),
            Number::U16(v) => s.serialize_u16(v),
            Number::U32(v) => s.serialize_u32(v),
            Number::U64(v) => s.serialize_u64(v),
            Number::I8(v) => s.serialize_i8(v),
            Number::I16(v) => s.serialize_i16(v),
            Number::I32(v) => s.serialize_i32(v),
            Number::I64(v) => s.serialize_i64(v),
            Number::F32(v) => s.serialize_f32(v),
            Number::F64(v) => s.serialize_f64(v),
        }
    }
}
