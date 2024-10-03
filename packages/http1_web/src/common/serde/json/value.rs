use std::fmt::Display;

use crate::common::serde::serialize::{MapSerializer, Serialize};

use super::map::OrderedMap;

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

#[derive(Debug, Clone)]
pub enum JsonValue {
    Number(Number),
    String(String),
    Bool(bool),
    Array(Vec<JsonValue>),
    Object(OrderedMap<String, JsonValue>),
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

    pub fn as_map(&self) -> Option<&OrderedMap<String, JsonValue>> {
        match self {
            JsonValue::Object(map) => Some(map),
            _ => None,
        }
    }
}

impl Serialize for Number {
    fn serialize<S: crate::common::serde::serialize::Serializer>(
        &self,
        serializer: S,
    ) -> Result<(), S::Err> {
        match self {
            Number::Float(f) => serializer.serialize_f64(*f),
            Number::UInteger(u) => serializer.serialize_u128(*u),
            Number::Integer(i) => serializer.serialize_i128(*i),
        }
    }
}

impl Serialize for JsonValue {
    fn serialize<S: crate::common::serde::serialize::Serializer>(
        &self,
        serializer: S,
    ) -> Result<(), S::Err> {
        match self {
            JsonValue::Number(number) => number.serialize(serializer),
            JsonValue::String(s) => serializer.serialize_str(s),
            JsonValue::Bool(b) => serializer.serialize_bool(*b),
            JsonValue::Array(vec) => serializer.serialize_vec(vec),
            JsonValue::Object(obj) => {
                let mut map = serializer.serialize_map()?;

                for (key, value) in obj.iter() {
                    map.serialize_entry(key, value)?;
                }

                map.end()?;
                Ok(())
            }
            JsonValue::Null => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonValue, Number};
    use crate::common::serde::json::{map::OrderedMap, to_pretty_string, to_string};

    #[test]
    fn should_serialize_number() {
        let f = JsonValue::Number(Number::Float(0.5));
        let u = JsonValue::Number(Number::UInteger(102398));
        let i = JsonValue::Number(Number::Integer(-1328));

        assert_eq!(to_string(&f).unwrap(), "0.5");
        assert_eq!(to_string(&u).unwrap(), "102398");
        assert_eq!(to_string(&i).unwrap(), "-1328");
    }

    #[test]
    fn should_serialize_string() {
        let s = JsonValue::String(String::from("Hello, world!"));
        assert_eq!(to_string(&s).unwrap(), "\"Hello, world!\"");
    }

    #[test]
    fn should_serialize_bool() {
        assert_eq!(to_string(&JsonValue::Bool(true)).unwrap(), "true");
        assert_eq!(to_string(&JsonValue::Bool(false)).unwrap(), "false");
    }

    #[test]
    fn should_serialize_array() {
        let array = JsonValue::Array(vec![
            JsonValue::Number(Number::Float(1.23)),
            JsonValue::Bool(true),
            JsonValue::String(String::from("Test")),
        ]);

        // Compact format
        assert_eq!(to_string(&array).unwrap(), "[1.23,true,\"Test\"]");

        // Pretty-printed format
        assert_eq!(
            to_pretty_string(&array).unwrap(),
            "[\n 1.23,\n true,\n \"Test\"\n]"
        );
    }

    #[test]
    fn should_serialize_object() {
        let mut map = OrderedMap::new();
        map.insert(
            String::from("number"),
            JsonValue::Number(Number::UInteger(123)),
        );
        map.insert(
            String::from("string"),
            JsonValue::String(String::from("Hello")),
        );
        map.insert(String::from("boolean"), JsonValue::Bool(false));

        let object = JsonValue::Object(map);

        // Compact format
        let expected_compact = "{\"number\":123,\"string\":\"Hello\",\"boolean\":false}";
        assert_eq!(to_string(&object).unwrap(), expected_compact);

        // Pretty-printed format
        let expected_pretty =
            "{\n \"number\": 123,\n \"string\": \"Hello\",\n \"boolean\": false\n}";
        assert_eq!(to_pretty_string(&object).unwrap(), expected_pretty);
    }

    #[test]
    fn should_serialize_null() {
        let null = JsonValue::Null;
        assert_eq!(to_string(&null).unwrap(), "null");
    }
}
