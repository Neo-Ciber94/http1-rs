use crate::common::serde::{
    de::{Deserialize, Deserializer},
    impossible::Impossible,
    ser::{MapSerializer, SequenceSerializer, Serialize, Serializer},
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

pub struct JsonValueDeserializer(pub JsonValue);
impl Deserializer for JsonValueDeserializer {
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }
}
