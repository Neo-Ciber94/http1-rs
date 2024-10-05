use crate::common::serde::{
    de::{Deserialize, Deserializer, Error},
    expected::Expected,
    impossible::Impossible,
    ser::{MapSerializer, SequenceSerializer, Serialize, Serializer},
    visitor::{MapAccess, SeqAccess, Visitor},
};

use super::{map::OrderedMap, number::Number, ser::JsonSerializationError};

#[derive(Debug, Clone)]
pub enum JsonValue {
    Number(Number),
    String(String),
    Bool(bool),
    Array(Vec<JsonValue>),
    Object(OrderedMap<String, JsonValue>),
    Null,
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonValue::Number(a), JsonValue::Number(b)) => a == b,
            (JsonValue::String(a), JsonValue::String(b)) => a == b,
            (JsonValue::Bool(a), JsonValue::Bool(b)) => a == b,
            (JsonValue::Array(a), JsonValue::Array(b)) => a == b,
            (JsonValue::Object(a), JsonValue::Object(b)) => a.iter().eq(b.iter()),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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
    fn serialize<S: crate::common::serde::ser::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self {
            Number::Float(f) => serializer.serialize_f64(*f),
            Number::UInteger(u) => serializer.serialize_u128(*u),
            Number::Integer(i) => serializer.serialize_i128(*i),
        }
    }
}

impl Serialize for JsonValue {
    fn serialize<S: crate::common::serde::ser::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
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

                map.end()
            }
            JsonValue::Null => serializer.serialize_none(),
        }
    }
}

pub struct JsonValueSerializer;
impl Serializer for JsonValueSerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;
    type Seq = Impossible<JsonValue, JsonSerializationError>;
    type Map = Impossible<JsonValue, JsonSerializationError>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Null)
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Bool(value))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::String(value.to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Null)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        todo!()
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        todo!()
    }
}

pub struct JsonArraySerializer(Vec<JsonValue>);
impl SequenceSerializer for JsonArraySerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;

    fn serialize_element<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err> {
        let mut arr = vec![];
        let json_value = value.serialize(JsonValueSerializer)?;
        arr.push(json_value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Array(self.0))
    }
}

pub struct JsonObjectSerializer(OrderedMap<String, JsonValue>);
impl MapSerializer for JsonObjectSerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        let k = key.serialize(JsonValueSerializer)?;
        let v = value.serialize(JsonValueSerializer)?;

        let JsonValue::String(s) = k else {
            return Err(JsonSerializationError::Other(format!(
                "json object keys should be strings"
            )));
        };

        self.0.insert(s, v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Object(self.0))
    }
}

impl Deserializer for JsonValue {
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Null => visitor.visit_unit(),
            _ => Err(Error::custom("expected unit")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Bool(value) => visitor.visit_bool(value),
            _ => Err(Error::custom("expected boolean")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_u8().ok_or_else(|| Error::custom("expected u8"))?;
                visitor.visit_u8(n)
            }
            _ => Err(Error::custom("expected u8")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_u16()
                    .ok_or_else(|| Error::custom("expected u16"))?;
                visitor.visit_u16(n)
            }
            _ => Err(Error::custom("expected u16")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_u32()
                    .ok_or_else(|| Error::custom("expected u32"))?;
                visitor.visit_u32(n)
            }
            _ => Err(Error::custom("expected u32")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_u64()
                    .ok_or_else(|| Error::custom("expected u64"))?;
                visitor.visit_u64(n)
            }
            _ => Err(Error::custom("expected u64")),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_u128()
                    .ok_or_else(|| Error::custom("expected u128"))?;
                visitor.visit_u128(n)
            }
            _ => Err(Error::custom("expected u128")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_i8().ok_or_else(|| Error::custom("expected i8"))?;
                visitor.visit_i8(n)
            }
            _ => Err(Error::custom("expected i8")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_i16()
                    .ok_or_else(|| Error::custom("expected i16"))?;
                visitor.visit_i16(n)
            }
            _ => Err(Error::custom("expected i16")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_i32()
                    .ok_or_else(|| Error::custom("expected i32"))?;
                visitor.visit_i32(n)
            }
            _ => Err(Error::custom("expected i32")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_i64()
                    .ok_or_else(|| Error::custom("expected i64"))?;
                visitor.visit_i64(n)
            }
            _ => Err(Error::custom("expected i64")),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_i128()
                    .ok_or_else(|| Error::custom("expected i128"))?;
                visitor.visit_i128(n)
            }
            _ => Err(Error::custom("expected i128")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_f32()
                    .ok_or_else(|| Error::custom("expected f32"))?;
                visitor.visit_f32(n)
            }
            _ => Err(Error::custom("expected f32")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_f64()
                    .ok_or_else(|| Error::custom("expected f64"))?;
                visitor.visit_f64(n)
            }
            _ => Err(Error::custom("expected f64")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::String(mut value) => {
                if value.is_empty() {
                    return Err(Error::custom("expected char but was empty string"));
                }

                if value.len() > 1 {
                    return Err(Error::custom("expected char but was string"));
                }

                let c = value.pop().unwrap();
                visitor.visit_char(c)
            }
            _ => Err(Error::custom("expected char")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::String(value) => visitor.visit_string(value),
            _ => Err(Error::custom("expected string")),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Array(value) => visitor.visit_seq(JsonSeqAccess(value.into_iter())),
            _ => Err(Error::custom("expected array")),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        match self {
            JsonValue::Object(value) => visitor.visit_map(JsonObjectAccess(value.into_iter())),
            _ => Err(Error::custom("expected object")),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        match self {
            JsonValue::Number(number) => match number {
                Number::Float(f) => visitor.visit_f64(f),
                Number::UInteger(u) => visitor.visit_u128(u),
                Number::Integer(i) => visitor.visit_i128(i),
            },
            JsonValue::String(s) => visitor.visit_string(s),
            JsonValue::Bool(value) => visitor.visit_bool(value),
            JsonValue::Array(vec) => visitor.visit_seq(JsonSeqAccess(vec.into_iter())),
            JsonValue::Object(ordered_map) => {
                visitor.visit_map(JsonObjectAccess(ordered_map.into_iter()))
            }
            JsonValue::Null => visitor.visit_none(),
        }
    }
    
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor {
        match self {
            JsonValue::Null => visitor.visit_none(),
            s => s.deserialize_any(visitor)
        }
    }
}

pub struct JsonSeqAccess(pub std::vec::IntoIter<JsonValue>);
impl SeqAccess for JsonSeqAccess {
    fn next_element<T: crate::common::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<T>, Error> {
        match self.0.next() {
            Some(x) => {
                let value = T::deserialize(x)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

pub struct JsonObjectAccess<I: Iterator<Item = (String, JsonValue)>>(pub I);
impl<I: Iterator<Item = (String, JsonValue)>> MapAccess for JsonObjectAccess<I> {
    fn next_entry<
        K: crate::common::serde::de::Deserialize,
        V: crate::common::serde::de::Deserialize,
    >(
        &mut self,
    ) -> Result<Option<(K, V)>, Error> {
        match self.0.next() {
            Some((k, v)) => {
                let key = K::deserialize(JsonValue::String(k))?;
                let value = V::deserialize(v)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }
}

impl Deserialize for JsonValue {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        struct ValueVisitor;
        impl Visitor for ValueVisitor {
            type Value = JsonValue;

            fn visit_unit(self) -> Result<Self::Value, Error> {
                Ok(JsonValue::Null)
            }

            fn visit_bool(self, value: bool) -> Result<Self::Value, Error> {
                Ok(JsonValue::Bool(value))
            }

            fn visit_u128(self, value: u128) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_i128(self, value: i128) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_none(self) -> Result<Self::Value, Error> {
                Ok(JsonValue::Null)
            }

            fn visit_option<T: Deserializer>(self, value: Option<T>) -> Result<Self::Value, Error> {
                match value {
                    Some(x) => x.deserialize_any(ValueVisitor),
                    None => self.visit_none(),
                }
            }

            fn visit_string(self, value: String) -> Result<Self::Value, Error> {
                Ok(JsonValue::String(value))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Expected for JsonValue {
    fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result {
        match self {
            JsonValue::Number(number) => number.expected(f, expected),
            JsonValue::String(value) => value.expected(f, expected),
            JsonValue::Bool(value) => value.expected(f, expected),
            JsonValue::Array(vec) => vec.expected(f, expected),
            JsonValue::Object(_) => write!(f, "expected `{expected}` but was object"),
            JsonValue::Null => write!(f, "expected `{expected}` but was null"),
        }
    }
}
