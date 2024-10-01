use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Number(Number),
    String(String),
    Bool(bool),
    Array(Vec<JsonValue>),
    Map(HashMap<String, JsonValue>),
    Null,
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Map(map) => Some(map),
            _ => None,
        }
    }
}
