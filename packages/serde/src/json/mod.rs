use std::io::{Read, Write};

use de::JsonDeserializer;
use formatter::{CompactFormatter, PrettyFormatter};
use ser::{JsonSerializationError, JsonSerializer};
use value::{JsonValue, JsonValueSerializer};

use super::{
    de::{Deserialize, Error},
    ser::Serialize,
};

pub mod de;
pub mod formatter;
pub mod number;
pub mod ser;
pub mod value;

// Serialize

/// Serialize the value of type `T` to a writer.
pub fn to_writer<W: Write, T: Serialize>(
    mut writer: W,
    value: &T,
) -> Result<(), JsonSerializationError> {
    let mut serializer = JsonSerializer::new(&mut writer, CompactFormatter);
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Serialize the value of type `T` to a writer formatted.
pub fn to_pretty_writer<W: Write, T: Serialize>(
    mut writer: W,
    value: &T,
) -> Result<(), JsonSerializationError> {
    let mut serializer = JsonSerializer::new(&mut writer, PrettyFormatter::new());
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Serialize a value of type `T` to bytes.
pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    to_writer(&mut buf, value)?;
    Ok(buf)
}

/// Serialize a value of type `T` to formatted JSON bytes.
pub fn to_pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    to_pretty_writer(&mut buf, value)?;
    Ok(buf)
}

/// Serialize a value of type `T` to a string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let bytes = to_bytes(value)?;
    String::from_utf8(bytes).map_err(|err| JsonSerializationError::Other(err.to_string()))
}

/// Serialize a value of type `T` to a formatted JSON string.
pub fn to_pretty_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let bytes = to_pretty_bytes(value)?;
    String::from_utf8(bytes).map_err(|err| JsonSerializationError::Other(err.to_string()))
}

/// Serialize a value of type `T` to a `JsonValue`.
pub fn to_value<T>(value: &T) -> Result<JsonValue, JsonSerializationError>
where
    T: Serialize,
{
    value.serialize(JsonValueSerializer)
}

// Deserialize

/// Deserialize a reader to a value of type `T`.
pub fn from_reader<T, R>(reader: R) -> Result<T, Error>
where
    T: Deserialize,
    R: Read,
{
    T::deserialize(JsonDeserializer::new(reader))
}

/// Deserialize bytes to a value of type `T`.
pub fn from_bytes<T>(bytes: impl AsRef<[u8]>) -> Result<T, Error>
where
    T: Deserialize,
{
    from_reader(bytes.as_ref())
}

/// Deserialize a string to a value of type `T`.
pub fn from_str<T>(str: impl AsRef<str>) -> Result<T, Error>
where
    T: Deserialize,
{
    from_reader(str.as_ref().as_bytes())
}

/// Deserialize a `JsonValue` to a value of type `T`.
pub fn from_value<T>(value: JsonValue) -> Result<T, Error>
where
    T: Deserialize,
{
    T::deserialize(value)
}
