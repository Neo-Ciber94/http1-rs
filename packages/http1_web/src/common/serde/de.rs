use std::fmt::Display;

use super::visitor::Visitor;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    Unexpected(Unexpected),
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Error {
    pub fn custom(msg: impl Into<String>) -> Self {
        Error::Custom(msg.into())
    }

    pub fn error<E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(error: E) -> Self {
        Error::Other(error.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "{msg}"),
            Error::Unexpected(unexpected) => write!(f, "unexpected {unexpected}"),
            Error::Other(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Unexpected {
    Bool(bool),
    Char(char),
    Unsigned(u128),
    Signed(i128),
    Float(f64),
    Str(String),
    Unit,
    Option,
    Seq,
    Map,
}

impl Display for Unexpected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unexpected::Bool(value) => write!(f, "boolean `{value}`"),
            Unexpected::Char(value) => write!(f, "char `{value}`"),
            Unexpected::Unsigned(value) => write!(f, "unsigned integer `{value}`"),
            Unexpected::Signed(value) => write!(f, "signed integer `{value}`"),
            Unexpected::Float(value) => write!(f, "float `{value}`"),
            Unexpected::Str(value) => write!(f, "string `{value}`"),
            Unexpected::Unit => write!(f, "unit type"),
            Unexpected::Option => write!(f, "option type"),
            Unexpected::Seq => write!(f, "sequence"),
            Unexpected::Map => write!(f, "map"),
        }
    }
}

pub trait Deserializer {
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;
}

pub trait Deserialize: Sized {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error>;
}
