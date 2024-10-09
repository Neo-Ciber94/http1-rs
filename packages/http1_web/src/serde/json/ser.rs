use std::{fmt::Display, io::Write};

use crate::common::serde::{
    impossible::Impossible,
    ser::{MapSerializer, SequenceSerializer, Serialize, Serializer},
};

use super::formatter::Formatter;

#[derive(Debug)]
pub enum JsonSerializationError {
    Other(String),
    IO(std::io::Error),
}

impl From<std::io::Error> for JsonSerializationError {
    fn from(value: std::io::Error) -> Self {
        JsonSerializationError::IO(value)
    }
}

impl Display for JsonSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSerializationError::Other(msg) => write!(f, "{msg}"),
            JsonSerializationError::IO(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for JsonSerializationError {}

#[derive(Debug, Clone)]
pub struct JsonSerializer<W, F> {
    writer: W,
    formatter: F,
}

impl JsonSerializer<(), ()> {
    pub fn new<W, F>(writer: W, formatter: F) -> JsonSerializer<W, F>
    where
        W: Write,
        F: Formatter<W>,
    {
        JsonSerializer { writer, formatter }
    }
}

impl<'a, W, F> Serializer for &'a mut JsonSerializer<W, F>
where
    W: Write,
    F: Formatter<W>,
{
    type Ok = ();
    type Err = JsonSerializationError;
    type Seq = JsonSequenceSerializer<'a, W, F>;
    type Map = JsonMapSerializer<'a, W, F>;

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_bool(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_str(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        self.formatter.write_array_start(&mut self.writer)?;

        Ok(JsonSequenceSerializer {
            serializer: self,
            count: 0,
        })
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        self.formatter.write_object_start(&mut self.writer)?;

        Ok(JsonMapSerializer {
            serializer: self,
            count: 0,
        })
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_null(&mut self.writer)?;
        Ok(())
    }

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<Self::Ok, Self::Err> {
        let mut seq = self.serialize_sequence()?;

        for x in value {
            seq.serialize_element(x)?;
        }

        seq.end()?;

        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        self.formatter.write_null(&mut self.writer)?;
        Ok(())
    }
}

pub struct JsonSequenceSerializer<'a, W, F> {
    serializer: &'a mut JsonSerializer<W, F>,
    count: usize,
}

impl<'a, W, F> SequenceSerializer for JsonSequenceSerializer<'a, W, F>
where
    W: Write,
    F: Formatter<W>,
{
    type Ok = ();
    type Err = JsonSerializationError;

    fn serialize_element<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err> {
        self.serializer
            .formatter
            .write_array_element_begin(&mut self.serializer.writer, self.count == 0)?;

        value.serialize(&mut (*self.serializer))?;

        self.serializer
            .formatter
            .write_array_element_end(&mut self.serializer.writer)?;

        self.count += 1;

        Ok(())
    }

    fn end(self) -> Result<(), Self::Err> {
        self.serializer
            .formatter
            .write_array_end(&mut self.serializer.writer)?;
        Ok(())
    }
}

pub struct JsonMapSerializer<'a, W, F> {
    serializer: &'a mut JsonSerializer<W, F>,
    count: usize,
}

impl<'a, W, F> MapSerializer for JsonMapSerializer<'a, W, F>
where
    W: Write,
    F: Formatter<W>,
{
    type Ok = ();
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        // Key
        self.serializer
            .formatter
            .write_object_key_begin(&mut self.serializer.writer, self.count == 0)?;

        key.serialize(MapKeySerializer {
            serializer: self.serializer,
        })?;

        self.serializer
            .formatter
            .write_object_key_end(&mut self.serializer.writer)?;

        // Value
        self.serializer
            .formatter
            .write_object_value_begin(&mut self.serializer.writer)?;

        value.serialize(&mut (*self.serializer))?;

        self.serializer
            .formatter
            .write_object_value_end(&mut self.serializer.writer)?;

        self.count += 1;

        Ok(())
    }

    fn end(self) -> Result<(), Self::Err> {
        self.serializer
            .formatter
            .write_object_end(&mut self.serializer.writer)?;
        Ok(())
    }
}

struct MapKeySerializer<'a, W, F> {
    serializer: &'a mut JsonSerializer<W, F>,
}

impl<'a, W: Write, F: Formatter<W>> Serializer for MapKeySerializer<'a, W, F> {
    type Ok = ();
    type Err = JsonSerializationError;
    type Seq = Impossible<Self::Ok, Self::Err>;
    type Map = Impossible<Self::Ok, Self::Err>;

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_str(self, value: &str) -> Result<(), Self::Err> {
        self.serializer
            .formatter
            .write_str(&mut self.serializer.writer, value)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_slice<T: Serialize>(self, _value: &[T]) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Err> {
        Err(map_key_error())
    }
}

fn map_key_error() -> JsonSerializationError {
    JsonSerializationError::Other("Keys can only be serialized to string".into())
}

#[cfg(test)]
mod tests {
    use crate::common::serde::json::{
        map::OrderedMap, number::Number, to_pretty_string, to_string, value::JsonValue,
    };

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
            "[\n  1.23,\n  true,\n  \"Test\"\n]"
        );
    }

    #[test]
    fn should_serialize_tuple() {
        // Arity 1 tuple
        assert_eq!(to_string(&(true,)).unwrap(), "[true]");

        // Arity 2 tuple
        assert_eq!(to_string(&(true, -12)).unwrap(), "[true,-12]");

        // Arity 3 tuple
        assert_eq!(
            to_string(&(true, -12, String::from("hello"))).unwrap(),
            "[true,-12,\"hello\"]"
        );

        // Arity 4 tuple
        assert_eq!(
            to_string(&(true, -12, String::from("hello"), 3.14)).unwrap(),
            "[true,-12,\"hello\",3.14]"
        );
        // Arity 5 tuple
        assert_eq!(
            to_string(&(true, -12, String::from("hello"), 3.14, 42u8)).unwrap(),
            "[true,-12,\"hello\",3.14,42]"
        );

        // Arity 6 tuple
        assert_eq!(
            to_string(&(true, -12, String::from("hello"), 3.14, 42u8, 'a')).unwrap(),
            "[true,-12,\"hello\",3.14,42,\"a\"]"
        );

        // Arity 7 tuple
        assert_eq!(
            to_string(&(true, -12, String::from("hello"), 3.14, 42u8, 'a', false)).unwrap(),
            "[true,-12,\"hello\",3.14,42,\"a\",false]"
        );

        // Arity 8 tuple
        assert_eq!(
            to_string(&(
                true,
                -12,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                false,
                0.01f32
            ))
            .unwrap(),
            "[true,-12,\"hello\",3.14,42,\"a\",false,0.01]"
        );

        // Arity 9 tuple
        assert_eq!(
            to_string(&(
                true,
                -12,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                false,
                0.1f32,
                99i64
            ))
            .unwrap(),
            "[true,-12,\"hello\",3.14,42,\"a\",false,0.1,99]"
        );

        // Arity 10 tuple
        assert_eq!(
            to_string(&(
                true,
                -12,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                false,
                0.1f32,
                99i64,
                255u16
            ))
            .unwrap(),
            "[true,-12,\"hello\",3.14,42,\"a\",false,0.1,99,255]"
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
            "{\n  \"number\": 123,\n  \"string\": \"Hello\",\n  \"boolean\": false\n}";
        assert_eq!(to_pretty_string(&object).unwrap(), expected_pretty);
    }

    #[test]
    fn should_serialize_null() {
        let null = JsonValue::Null;
        assert_eq!(to_string(&null).unwrap(), "null");
    }
}
