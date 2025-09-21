use crate::number::Number;
use alloc::boxed::Box;
use alloc::string::ToString;
use core::convert::Infallible;
use core::fmt;

#[derive(Debug)]
pub struct TryFromNumberError {
    source: Box<dyn core::error::Error + Send + Sync>,
}

impl fmt::Display for TryFromNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl core::error::Error for TryFromNumberError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.source.source()
    }
}

impl From<core::num::TryFromIntError> for TryFromNumberError {
    fn from(value: core::num::TryFromIntError) -> Self {
        TryFromNumberError {
            source: value.to_string().into(),
        }
    }
}

impl From<Infallible> for TryFromNumberError {
    fn from(value: Infallible) -> Self {
        TryFromNumberError {
            source: Box::new(value),
        }
    }
}

macro_rules! from_impl {
    ($from: ty, $map: ident) => {
        impl From<$from> for Number {
            fn from(from: $from) -> Number {
                Number::$map(from)
            }
        }

        impl TryFrom<Number> for $from {
            type Error = TryFromNumberError;
            fn try_from(number: Number) -> Result<$from, Self::Error> {
                use Number::*;
                let ret = match number {
                    U8(v) => v.try_into()?,
                    I8(v) => v.try_into()?,
                    U16(v) => v.try_into()?,
                    I16(v) => v.try_into()?,
                    U32(v) => v.try_into()?,
                    I32(v) => v.try_into()?,
                    U64(v) => v.try_into()?,
                    I64(v) => v.try_into()?,
                    _ => {
                        return Err(TryFromNumberError {
                            source: "cannot convert from float".into(),
                        });
                    }
                };

                Ok(ret)
            }
        }
    };

    (float $from: ty, $map: ident) => {
        impl From<$from> for Number {
            fn from(from: $from) -> Number {
                Number::$map(from)
            }
        }

        impl TryFrom<Number> for $from {
            type Error = TryFromNumberError;
            fn try_from(number: Number) -> Result<$from, Self::Error> {
                use Number::*;
                let ret = match number {
                    U8(v) => v.try_into()?,
                    I8(v) => v.try_into()?,
                    U16(v) => v.try_into()?,
                    I16(v) => v.try_into()?,
                    U32(v) => v.try_into()?,
                    I32(v) => v.try_into()?,
                    U64(v) => v.try_into()?,
                    I64(v) => v.try_into()?,
                    _ => {
                        return Err(TryFromNumberError {
                            source: "cannot convert from float".into(),
                        });
                    }
                };

                Ok(ret)
            }
        }
    };
}

from_impl!(u8, U8);
from_impl!(i8, I8);
from_impl!(u16, U16);
from_impl!(i16, I16);
from_impl!(i32, I32);
from_impl!(u32, U32);
from_impl!(i64, I64);
from_impl!(u64, U64);

impl From<f32> for Number {
    fn from(from: f32) -> Number {
        Number::F32(from)
    }
}

impl TryFrom<Number> for f32 {
    type Error = TryFromNumberError;
    fn try_from(number: Number) -> Result<f32, Self::Error> {
        use Number::*;
        let ret = match number {
            U8(v) => v.try_into()?,
            I8(v) => v.try_into()?,
            U16(v) => v.try_into()?,
            I16(v) => v.try_into()?,
            F32(v) => v.try_into()?,
            F64(v) => {
                let x = v as f32;
                if x.is_finite() == v.is_finite() {
                    x
                } else {
                    return Err(TryFromNumberError {
                        source: "f32 overflow during conversion".into(),
                    });
                }
            }
            _ => {
                return Err(TryFromNumberError {
                    source: "cannot convert from integer".into(),
                });
            }
        };

        Ok(ret)
    }
}

impl From<f64> for Number {
    fn from(from: f64) -> Number {
        Number::F64(from)
    }
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number::U64(value as _)
    }
}

impl TryFrom<Number> for f64 {
    type Error = TryFromNumberError;
    fn try_from(number: Number) -> Result<f64, Self::Error> {
        use Number::*;
        let ret = match number {
            U8(v) => v.try_into()?,
            I8(v) => v.try_into()?,
            U16(v) => v.try_into()?,
            I16(v) => v.try_into()?,
            U32(v) => v.try_into()?,
            I32(v) => v.try_into()?,
            F32(v) => v as f64,
            F64(v) => v,
            _ => {
                return Err(TryFromNumberError {
                    source: "cannot convert from integer".into(),
                });
            }
        };

        Ok(ret)
    }
}
