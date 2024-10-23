use std::fmt::Display;

use crate::serde::expected::Expected;

#[derive(Clone, Copy, Debug, PartialOrd)]
pub enum Number {
    Float(f64),
    UInteger(u128),
    Integer(i128),
}

impl Number {
    pub fn is_float(&self) -> bool {
        matches!(self, Number::Float(_))
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Number::UInteger(_))
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(self, Number::Integer(_))
    }

    // Floats
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Number::Float(f) => (*f as f32).try_into().ok(),
            Number::Integer(i) => (*i as f32).try_into().ok(), // Convert signed integer to float
            Number::UInteger(u) => (*u as f32).try_into().ok(), // Convert unsigned integer to float
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Number::Float(f) => Some(*f),
            Number::Integer(i) => (*i as f64).try_into().ok(),
            Number::UInteger(u) => (*u as f64).try_into().ok(),
        }
    }

    // Signed Integers
    pub fn as_i8(&self) -> Option<i8> {
        match self {
            Number::Integer(i) => (*i).try_into().ok(),
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Float(f) => (*f as i8).try_into().ok(), // Convert float to signed integer
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            Number::Integer(i) => (*i).try_into().ok(),
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Float(f) => (*f as i16).try_into().ok(),
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Number::Integer(i) => (*i).try_into().ok(),
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Float(f) => (*f as i32).try_into().ok(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number::Integer(i) => (*i).try_into().ok(),
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Float(f) => (*f as i64).try_into().ok(),
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match self {
            Number::Integer(i) => Some(*i),
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Float(f) => (*f as i128).try_into().ok(),
        }
    }

    // Unsigned Integers
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Integer(i) => (*i).try_into().ok(),
            Number::Float(f) => (*f as u8).try_into().ok(), // Convert float to unsigned integer
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Integer(i) => (*i).try_into().ok(),
            Number::Float(f) => (*f as u16).try_into().ok(),
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Integer(i) => (*i).try_into().ok(),
            Number::Float(f) => (*f as u32).try_into().ok(),
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Number::UInteger(u) => (*u).try_into().ok(),
            Number::Integer(i) => (*i).try_into().ok(),
            Number::Float(f) => (*f as u64).try_into().ok(),
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        match self {
            Number::UInteger(u) => Some(*u),
            Number::Integer(i) => (*i).try_into().ok(),
            Number::Float(f) => (*f as u128).try_into().ok(),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::UInteger(a), Self::UInteger(b)) => a == b,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Float(a), Self::UInteger(b)) => (*a).is_finite() && *a == *b as f64,
            (Self::Float(a), Self::Integer(b)) => (*a).is_finite() && *a == *b as f64,
            (Self::UInteger(a), Self::Float(b)) => *a as f64 == *b,
            (Self::UInteger(a), Self::Integer(b)) => (*b >= 0) && *a == (*b as u128),
            (Self::Integer(a), Self::UInteger(b)) => (*a >= 0) && (*a as u128) == *b,
            (Self::Integer(a), Self::Float(b)) => (*a as f64) == *b,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Float(x) => write!(f, "{x}"),
            Number::UInteger(x) => write!(f, "{x}"),
            Number::Integer(x) => write!(f, "{x}"),
        }
    }
}

macro_rules! impl_from_number {
    (unsigned = [$($U:ty),*], signed = [$($I:ty),*], float = [$($F:ty),*]) => {
        $(
            impl From<$U> for Number {
                fn from(value: $U) -> Self {
                    Number::UInteger(value.into())
                }
            }
        )*

        $(
            impl From<$I> for Number {
                fn from(value: $I) -> Self {
                    Number::Integer(value.into())
                }
            }
        )*

        $(
            impl From<$F> for Number {
                fn from(value: $F) -> Self {
                    Number::Float(value.into())
                }
            }
        )*
    };
}

impl_from_number!(
    unsigned = [u8, u16, u32, u64, u128],
    signed = [i8, i16, i32, i64, i128],
    float = [f32, f64]
);

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number::UInteger(value as u128)
    }
}

impl From<isize> for Number {
    fn from(value: isize) -> Self {
        Number::Integer(value as i128)
    }
}

impl Expected for Number {
    fn expected(&self) -> &'static str {
        match self {
            Number::Float(_) => "float",
            Number::UInteger(_) => "unsigned integer",
            Number::Integer(_) => "signed integer",
        }
    }
}
