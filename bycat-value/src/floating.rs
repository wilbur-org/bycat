use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

// masks for the parts of the IEEE 754 float
const SIGN_MASK: u64 = 0x8000000000000000u64;
const EXP_MASK: u64 = 0x7ff0000000000000u64;
const MAN_MASK: u64 = 0x000fffffffffffffu64;

// canonical raw bit patterns (for hashing)
const CANONICAL_NAN_BITS: u64 = 0x7ff8000000000000u64;
const CANONICAL_ZERO_BITS: u64 = 0x0u64;

fn integer_decode_f32(f: f32) -> (u64, i16, i8) {
    let bits: u32 = f.to_bits();
    let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0x7fffff) << 1
    } else {
        (bits & 0x7fffff) | 0x800000
    };
    // Exponent bias + mantissa shift
    exponent -= 127 + 23;
    (mantissa as u64, exponent, sign)
}

fn integer_decode_f64(f: f64) -> (u64, i16, i8) {
    let bits: u64 = f.to_bits();
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    // Exponent bias + mantissa shift
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for f32 {}

    impl Sealed for f64 {}

    impl<'a, S> Sealed for &'a S where S: Sealed {}
}

pub trait Float: sealed::Sealed + PartialOrd {
    fn integer_decode(&self) -> (u64, i16, i8);
    fn is_nan(&self) -> bool;
}

impl Float for f32 {
    #[inline]
    fn integer_decode(&self) -> (u64, i16, i8) {
        integer_decode_f32(*self)
    }

    #[inline]
    #[allow(clippy::eq_op)]
    fn is_nan(&self) -> bool {
        self != self
    }
}

impl Float for f64 {
    #[inline]
    fn integer_decode(&self) -> (u64, i16, i8) {
        integer_decode_f64(*self)
    }

    #[inline]
    #[allow(clippy::eq_op)]
    fn is_nan(&self) -> bool {
        self != self
    }
}

impl<'a, F: Float> Float for &'a F {
    #[inline]
    fn integer_decode(&self) -> (u64, i16, i8) {
        (*self).integer_decode()
    }

    #[inline]
    fn is_nan(&self) -> bool {
        (*self).is_nan()
    }
}

#[inline]
pub fn cmp<F: Float>(left: F, right: F) -> Ordering {
    if left.is_nan() {
        if right.is_nan() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    } else {
        match left.partial_cmp(&right) {
            Some(o) => o,
            None => Ordering::Less,
        }
    }
}

#[inline]
pub fn eq<F: Float>(left: F, right: F) -> bool {
    if left.is_nan() {
        right.is_nan()
    } else {
        left == right
    }
}

#[inline]
pub fn hash<F: Float, H: Hasher>(f: &F, state: &mut H) {
    raw_double_bits(f).hash(state);
}

#[inline]
fn raw_double_bits<F: Float>(f: &F) -> u64 {
    if f.is_nan() {
        return CANONICAL_NAN_BITS;
    }

    let (man, exp, sign) = f.integer_decode();
    if man == 0 {
        return CANONICAL_ZERO_BITS;
    }

    let exp_u64 = exp as u16 as u64;
    let sign_u64 = (sign > 0) as u64;
    (man & MAN_MASK) | ((exp_u64 << 52) & EXP_MASK) | ((sign_u64 << 63) & SIGN_MASK)
}
