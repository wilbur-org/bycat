use crate::floating;
use core::{
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Copy)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::U8(l), Number::U8(r)) => l == r,
            (Number::I8(l), Number::I8(r)) => l == r,
            (Number::I16(l), Number::I16(r)) => l == r,
            (Number::U16(l), Number::U16(r)) => l == r,
            (Number::I32(l), Number::I32(r)) => l == r,
            (Number::U32(l), Number::U32(r)) => l == r,
            (Number::I64(l), Number::I64(r)) => l == r,
            (Number::U64(l), Number::U64(r)) => l == r,
            (Number::F32(l), Number::F32(r)) => floating::eq(l, r),
            (Number::F64(l), Number::F64(r)) => floating::eq(l, r),
            (l, r) => {
                if l.is_float() && r.is_float() {
                    let l = l.as_f64();
                    let r = r.as_f64();
                    floating::eq(l, r)
                } else {
                    l.as_u64() == r.as_u64()
                }
            }
        }
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (Number::U8(l), Number::U8(r)) => l.cmp(r),
            (Number::I8(l), Number::I8(r)) => l.cmp(r),
            (Number::I16(l), Number::I16(r)) => l.cmp(r),
            (Number::U16(l), Number::U16(r)) => l.cmp(r),
            (Number::I32(l), Number::I32(r)) => l.cmp(r),
            (Number::U32(l), Number::U32(r)) => l.cmp(r),
            (Number::I64(l), Number::I64(r)) => l.cmp(r),
            (Number::U64(l), Number::U64(r)) => l.cmp(r),
            (Number::F32(l), Number::F32(r)) => floating::cmp(l, r),
            (Number::F64(l), Number::F64(r)) => floating::cmp(l, r),
            (l, r) => {
                if l.is_float() && r.is_float() {
                    let l = l.as_f64();
                    let r = r.as_f64();
                    floating::cmp(l, r)
                } else {
                    l.as_u64().cmp(&r.as_u64())
                }
            }
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.is_float() {
            floating::hash(&self.as_f64(), state)
        } else {
            self.as_u64().hash(state)
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Number::U8(i) => write!(f, "{}", i),
            Number::I8(i) => write!(f, "{}", i),
            Number::U16(i) => write!(f, "{}", i),
            Number::I16(i) => write!(f, "{}", i),
            Number::I32(i) => write!(f, "{}", i),
            Number::U32(i) => write!(f, "{}", i),
            Number::I64(i) => write!(f, "{}", i),
            Number::U64(i) => write!(f, "{}", i),
            #[cfg(not(feature = "ryu"))]
            Number::F32(n) => write!(f, "{}", n),
            #[cfg(feature = "ryu")]
            Number::F32(n) => write!(f, "{}", ryu::Buffer::new().format(n)),
            #[cfg(not(feature = "ryu"))]
            Number::F64(n) => write!(f, "{}", n),
            #[cfg(feature = "ryu")]
            Number::F64(n) => write!(f, "{}", ryu::Buffer::new().format(n)),
        }
    }
}

macro_rules! as_method {
    ($method: ident, $is_method: ident, $variant: ident, $ty: ty) => {
        #[inline]
        pub fn $method(&self) -> $ty {
            match self {
                Self::$variant(i) => *i,
                n => n.as_u64() as $ty,
            }
        }

        #[inline]
        pub fn $is_method(&self) -> bool {
            match self {
                Self::$variant(_) => true,
                _ => false,
            }
        }
    };
}

impl Number {
    #[inline]
    pub fn as_u64(&self) -> u64 {
        match *self {
            Number::U8(i) => i as u64,
            Number::I8(i) => i as u64,
            Number::U16(i) => i as u64,
            Number::I16(i) => i as u64,
            Number::I32(i) => i as u64,
            Number::U32(i) => i as u64,
            Number::I64(i) => i as u64,
            Number::U64(i) => i,
            Number::F32(n) => n as u64,
            Number::F64(n) => n as u64,
        }
    }

    as_method!(as_i64, is_i64, I64, i64);
    as_method!(as_i8, is_i8, I8, i8);
    as_method!(as_u8, is_u8, U8, u8);
    as_method!(as_i16, is_i16, I16, i16);
    as_method!(as_u16, is_u16, U16, u16);
    as_method!(as_i32, is_i32, I32, i32);
    as_method!(as_u32, is_u32, U32, u32);

    #[inline]
    pub fn as_f32(&self) -> f32 {
        match *self {
            Number::U8(i) => i as f32,
            Number::I8(i) => i as f32,
            Number::U16(i) => i as f32,
            Number::I16(i) => i as f32,
            Number::I32(i) => i as f32,
            Number::U32(i) => i as f32,
            Number::I64(i) => i as f32,
            Number::U64(i) => i as f32,
            Number::F32(n) => n,
            Number::F64(n) => n as f32,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> f64 {
        match *self {
            Number::U8(i) => i as f64,
            Number::I8(i) => i as f64,
            Number::U16(i) => i as f64,
            Number::I16(i) => i as f64,
            Number::I32(i) => i as f64,
            Number::U32(i) => i as f64,
            Number::I64(i) => i as f64,
            Number::U64(i) => i as f64,
            Number::F32(n) => n as f64,
            Number::F64(n) => n,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Number::F32(_) | Number::F64(_))
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }
}

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}

macro_rules! arit_impl {
    ($ty: ident, $input: expr, $method: ident, $op: expr) => {
        $ty(v) => Number::$ty(expr!(v $op expr.$method()))
    };
    ($self: ident, $input: expr, $op: tt) => {
        {
            use Number::*;
            match $self {
                U8(v) => U8(expr!(v $op $input.as_u8())),
                I8(v) => I8(expr!(v $op $input.as_i8())),
                U16(v) => U16(expr!(v $op $input.as_u16())),
                I16(v) => I16(expr!(v $op $input.as_i16())),
                U32(v) => U32(expr!(v $op $input.as_u32())),
                I32(v) => I32(expr!(v $op $input.as_i32())),
                U64(v) => U64(expr!(v $op $input.as_u64())),
                I64(v) => I64(expr!(v $op $input.as_i64())),
                F32(v) => F32(expr!(v $op $input.as_f32())),
                F64(v) => F64(expr!(v $op $input.as_f64())),
            }
        }
    };
}

impl<V: Into<Number>> core::ops::Add<V> for Number {
    type Output = Number;
    fn add(self, rhs: V) -> Self::Output {
        arit_impl!(self, rhs.into(), +)
    }
}

impl<V: Into<Number>> core::ops::AddAssign<V> for Number {
    fn add_assign(&mut self, rhs: V) {
        *self = *self + rhs;
    }
}

impl<V: Into<Number>> core::ops::Sub<V> for Number {
    type Output = Number;
    fn sub(self, rhs: V) -> Self::Output {
        arit_impl!(self, rhs.into(), -)
    }
}

impl<V: Into<Number>> core::ops::SubAssign<V> for Number {
    fn sub_assign(&mut self, rhs: V) {
        *self = *self - rhs;
    }
}

impl<V: Into<Number>> core::ops::Mul<V> for Number {
    type Output = Number;
    fn mul(self, rhs: V) -> Self::Output {
        arit_impl!(self, rhs.into(), *)
    }
}

impl<V: Into<Number>> core::ops::MulAssign<V> for Number {
    fn mul_assign(&mut self, rhs: V) {
        *self = *self * rhs;
    }
}

impl<V: Into<Number>> core::ops::Div<V> for Number {
    type Output = Number;
    fn div(self, rhs: V) -> Self::Output {
        arit_impl!(self, rhs.into(), /)
    }
}

impl<V: Into<Number>> core::ops::DivAssign<V> for Number {
    fn div_assign(&mut self, rhs: V) {
        *self = *self / rhs;
    }
}
