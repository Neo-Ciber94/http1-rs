use std::{fmt::Display, io::Write};

use crate::common::serde::{
    impossible::Impossible,
    serialize::{MapSerializer, SequenceSerializer, Serialize, Serializer},
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

impl<'a, W, F> Serializer for &'a mut JsonSerializer<W, F>
where
    W: Write,
    F: Formatter<W>,
{
    type Err = JsonSerializationError;
    type Seq = JsonSequenceSerializer<'a, W, F>;
    type Map = JsonMapSerializer<'a, W, F>;

    fn serialize_i128(self, value: i128) -> Result<(), Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<(), Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Self::Err> {
        self.formatter.write_number(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_bool(self, value: bool) -> Result<(), Self::Err> {
        self.formatter.write_bool(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Self::Err> {
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
        Ok(JsonMapSerializer {
            serializer: self,
            count: 0,
        })
    }

    fn serialize_none(self) -> Result<(), Self::Err> {
        self.formatter.write_null(&mut self.writer)?;
        Ok(())
    }

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<(), Self::Err> {
        let mut seq = self.serialize_sequence()?;

        for x in value {
            seq.serialize_element(x)?;
        }

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
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        // Key
        self.serializer
            .formatter
            .write_object_key_begin(&mut self.serializer.writer)?;

        key.serialize(MapKeySerializer {
            serializer: self.serializer,
        })?;

        self.serializer
            .formatter
            .write_object_key_end(&mut self.serializer.writer)?;

        // Value
        self.serializer
            .formatter
            .write_object_value_begin(&mut self.serializer.writer, self.count == 0)?;

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
            .write_object_key_end(&mut self.serializer.writer)?;
        Ok(())
    }
}

struct MapKeySerializer<'a, W, F> {
    serializer: &'a mut JsonSerializer<W, F>,
}

impl<'a, W: Write, F: Formatter<W>> Serializer for MapKeySerializer<'a, W, F> {
    type Err = JsonSerializationError;
    type Seq = Impossible<Self::Err>;
    type Map = Impossible<Self::Err>;

    fn serialize_i128(self, _value: i128) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_u128(self, _value: u128) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_f64(self, _value: f64) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_bool(self, _value: bool) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_none(self) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_str(self, value: &str) -> Result<(), Self::Err> {
        self.serializer.writer.write(value.as_bytes())?;
        Ok(())
    }
}

fn map_key_error() -> JsonSerializationError {
    JsonSerializationError::Other(format!("Keys can only be serialized to string"))
}
