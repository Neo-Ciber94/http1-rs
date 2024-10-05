use std::io::{Read, Write};

use de::JsonDeserializer;
use formatter::{CompactFormatter, PrettyFormatter};
use ser::{JsonSerializationError, JsonSerializer};
use value::{JsonValue, JsonValueDeserializer, JsonValueSerializer};

use super::{
    de::{Deserialize, Error},
    ser::Serialize,
};

pub mod de;
pub mod formatter;
pub mod map;
pub mod number;
pub mod ser;
pub mod value;

// Serialize

pub fn to_writer<W: Write, T: Serialize>(
    mut writer: W,
    value: &T,
) -> Result<(), JsonSerializationError> {
    let mut serializer = JsonSerializer::new(&mut writer, CompactFormatter);
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn to_pretty_writer<W: Write, T: Serialize>(
    mut writer: W,
    value: &T,
) -> Result<(), JsonSerializationError> {
    let mut serializer = JsonSerializer::new(&mut writer, PrettyFormatter::new());
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    to_writer(&mut buf, value)?;
    Ok(buf)
}

pub fn to_pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    to_pretty_writer(&mut buf, value)?;
    Ok(buf)
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let bytes = to_bytes(value)?;
    String::from_utf8(bytes).map_err(|err| JsonSerializationError::Other(err.to_string()))
}

pub fn to_pretty_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let bytes = to_pretty_bytes(value)?;
    String::from_utf8(bytes).map_err(|err| JsonSerializationError::Other(err.to_string()))
}

pub fn to_value<T>(value: &T) -> Result<JsonValue, JsonSerializationError>
where
    T: Serialize,
{
    value.serialize(JsonValueSerializer)
}

// Deserialize

pub fn from_reader<T, R>(reader: R) -> Result<T, Error>
where
    T: Deserialize,
    R: Read,
{
    T::deserialize(JsonDeserializer::new(reader))
}

pub fn from_bytes<T, S>(bytes: S) -> Result<T, Error>
where
    T: Deserialize,
    S: AsRef<[u8]>,
{
    from_reader(bytes.as_ref())
}

pub fn from_str<T, S>(str: S) -> Result<T, Error>
where
    T: Deserialize,
    S: AsRef<str>,
{
    from_reader(str.as_ref().as_bytes())
}

pub fn from_value<T>(value: JsonValue) -> Result<T, Error>
where
    T: Deserialize,
{
    T::deserialize(JsonValueDeserializer(value))
}
