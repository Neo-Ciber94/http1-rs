use std::{collections::HashMap, fmt::Display};

use crate::common::serde::serialize::{MapSerializer, SequenceSerializer, Serialize, Serializer};

use super::value::{JsonValue, Number};

#[derive(Debug)]
pub enum JsonSerializationError {
    Other(String),
}

impl Display for JsonSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSerializationError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for JsonSerializationError {}

#[derive(Debug, Clone)]
pub struct JsonSerializer(JsonValue);

impl JsonSerializer {
    pub fn into_json(self) -> JsonValue {
        self.0
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self(JsonValue::Null)
    }
}

impl Serializer for JsonSerializer {
    type Err = JsonSerializationError;
    type Seq = JsonSequenceSerializer;
    type Map = JsonMapSerializer;

    fn serialize_i128(&mut self, value: i128) -> Result<(), Self::Err> {
        self.0 = JsonValue::Number(Number::Integer(value));
        Ok(())
    }

    fn serialize_u128(&mut self, value: u128) -> Result<(), Self::Err> {
        self.0 = JsonValue::Number(Number::UInteger(value));
        Ok(())
    }

    fn serialize_f64(&mut self, value: f64) -> Result<(), Self::Err> {
        self.0 = JsonValue::Number(Number::Float(value));
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Err> {
        self.0 = JsonValue::Bool(value);
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Err> {
        self.0 = JsonValue::String(value.to_owned());
        Ok(())
    }

    fn serialize_sequence(&mut self) -> Result<Self::Seq, Self::Err> {
        let seq_serializer = JsonSequenceSerializer::default();
        Ok(seq_serializer)
    }

    fn serialize_map(&mut self) -> Result<Self::Map, Self::Err> {
        let map_serializer = JsonMapSerializer::default();
        Ok(map_serializer)
    }
}

#[derive(Default, Debug, Clone)]
pub struct JsonSequenceSerializer(Vec<JsonValue>);

impl SequenceSerializer for JsonSequenceSerializer {
    type Err = JsonSerializationError;

    fn serialize_next<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err> {
        let mut serializer = JsonSerializer::default();
        value.serialize(&mut serializer)?;
        self.0.push(serializer.into_json());

        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct JsonMapSerializer(HashMap<String, JsonValue>);

impl MapSerializer for JsonMapSerializer {
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        let mut key_serializer = JsonSerializer::default();
        let mut value_serializer = JsonSerializer::default();

        key.serialize(&mut key_serializer)?;
        value.serialize(&mut value_serializer)?;

        let key_json = match key_serializer.into_json() {
            JsonValue::String(s) => s,
            v => {
                return Err(JsonSerializationError::Other(format!(
                    "Map keys should be string but was '{}'",
                    std::any::type_name_of_val(&v)
                )))
            }
        };

        let value_json = value_serializer.into_json();
        self.0.insert(key_json, value_json);

        Ok(())
    }
}
