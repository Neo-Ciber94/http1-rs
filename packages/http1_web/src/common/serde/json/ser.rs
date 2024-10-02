use std::{fmt::Display, io::Write};

use crate::common::serde::serialize::{MapSerializer, SequenceSerializer, Serialize, Serializer};

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
pub struct JsonSerializer<W> {
    writer: W,
}

impl<'a, W> Serializer for &'a mut JsonSerializer<W>
where
    W: Write,
{
    type Err = JsonSerializationError;
    type Seq = JsonSequenceSerializer<'a, W>;
    type Map = JsonMapSerializer<'a, W>;

    fn serialize_i128(self, value: i128) -> Result<(), Self::Err> {
        let s = value.to_string();
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<(), Self::Err> {
        let s = value.to_string();
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Self::Err> {
        let s = value.to_string();
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn serialize_bool(self, value: bool) -> Result<(), Self::Err> {
        if value {
            self.writer.write(b"true")?
        } else {
            self.writer.write(b"false")?
        };

        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Self::Err> {
        let s = format!("\"{value}\"");
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        self.writer.write(b"[")?;

        Ok(JsonSequenceSerializer {
            writer: &mut self.writer,
            count: 0,
        })
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Ok(JsonMapSerializer {
            writer: &mut self.writer,
        })
    }

    fn serialize_unit(self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_u8(self, value: u8) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_f32(self, value: f32) -> Result<(), Self::Err> {
        self.serialize_f64(value.into())
    }

    fn serialize_string(self, value: String) -> Result<(), Self::Err> {
        self.serialize_str(&value)
    }

    fn serialize_char(self, value: char) -> Result<(), Self::Err> {
        self.serialize_string(value.to_string())
    }

    fn serialize_option<T: Serialize>(self, value: Option<T>) -> Result<(), Self::Err> {
        match value {
            Some(x) => x.serialize(self),
            None => {
                self.writer.write(b"null")?;
                Ok(())
            }
        }
    }

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<(), Self::Err> {
        let mut seq = self.serialize_sequence()?;

        for x in value {
            seq.serialize_next(x)?;
        }

        Ok(())
    }

    fn serialize_array<T: Serialize, const N: usize>(self, value: [T; N]) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_vec<T: Serialize>(self, value: Vec<T>) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }
}

pub struct JsonSequenceSerializer<'a, W> {
    writer: &'a mut W,
    count: usize,
}

impl<'a, W: Write> SequenceSerializer for JsonSequenceSerializer<'a, W> {
    type Err = JsonSerializationError;

    fn serialize_next<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err> {
        if self.count > 0 {
            self.writer.write(b",")?;
        }

        self.count += 1;
        value.serialize(&mut JsonSerializer {
            writer: &mut self.writer,
        })?;

        Ok(())
    }

    fn end(self) -> Result<(), Self::Err> {
        self.writer.write(b"]")?;
        Ok(())
    }
}

pub struct JsonMapSerializer<'a, W> {
    writer: &'a mut W,
}

impl<'a, W: Write> MapSerializer for JsonMapSerializer<'a, W> {
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        key.serialize(MapKeySerializer {
            writer: &mut self.writer,
        })?;

        self.writer.write(b":")?;

        value.serialize(&mut JsonSerializer {
            writer: &mut self.writer,
        })?;

        Ok(())
    }

    fn end(self) -> Result<(), Self::Err> {
        self.writer.write(b"}")?;
        Ok(())
    }
}

struct MapKeySerializer<W> {
    writer: W,
}

fn map_key_error() -> JsonSerializationError {
    JsonSerializationError::Other(format!("Keys can only be serialized to string"))
}

struct Impossible;
impl MapSerializer for Impossible {
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn end(self) -> Result<(), Self::Err> {
        Err(map_key_error())
    }
}

impl SequenceSerializer for Impossible {
    type Err = JsonSerializationError;

    fn serialize_next<T: Serialize>(&mut self, _value: &T) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn end(self) -> Result<(), Self::Err> {
        Err(map_key_error())
    }
}

impl<W: Write> Serializer for MapKeySerializer<W> {
    type Err = JsonSerializationError;
    type Seq = Impossible;
    type Map = Impossible;

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

    fn serialize_option<T: Serialize>(self, _value: Option<T>) -> Result<(), Self::Err> {
        Err(map_key_error())
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Err(map_key_error())
    }

    fn serialize_str(mut self, value: &str) -> Result<(), Self::Err> {
        self.writer.write(value.as_bytes())?;
        Ok(())
    }
}
