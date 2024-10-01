use super::serialize::{Serialize, Serializer};

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum N {
    Float(f64),
    PosInteger(u128),
    NegInteger(i128),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Number(N);

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
    fn write_i128(&mut self, value: i128) {
        self.0 = JsonValue::Number(Number(N::NegInteger(value)));
    }

    fn write_u128(&mut self, value: u128) {
        self.0 = JsonValue::Number(Number(N::PosInteger(value)));
    }

    fn write_f64(&mut self, value: f64) {
        self.0 = JsonValue::Number(Number(N::Float(value)));
    }

    fn write_bool(&mut self, value: bool) {
        self.0 = JsonValue::Bool(value);
    }

    fn write_str(&mut self, value: &str) {
        self.0 = JsonValue::String(value.to_owned());
    }

    fn write_slice<T: Serialize>(&mut self, value: &[T]) {
        let mut vec = vec![];

        for item in value {
            let mut s = JsonSerializer::default();
            item.serialize(&mut s);
            vec.push(s.into_json());
        }

        self.0 = JsonValue::Array(vec);
    }

    fn write_map<E: IntoIterator<Item = (String, T)>, T: Serialize>(
        &mut self,
        data: E,
    ) {
        let mut map = HashMap::new();

        for (key, value) in data {
            let mut s = JsonSerializer::default();
            value.serialize(&mut s);
            map.insert(key, s.into_json());
        }

        self.0 = JsonValue::Map(map);
    }
}

struct Fruit {
    name: String,
    color: String,
}

impl Serialize for Fruit {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
       todo!()
    }
}
