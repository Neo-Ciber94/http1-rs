use formatter::{CompactFormatter, PrettyFormatter};
use ser::{JsonSerializationError, JsonSerializer};

use super::serialize::Serialize;

pub mod formatter;
pub mod map;
pub mod ser;
pub mod value;

pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    let mut serializer = JsonSerializer::new(&mut buf, CompactFormatter);
    value.serialize(&mut serializer)?;
    Ok(buf)
}

pub fn to_pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    let mut serializer = JsonSerializer::new(&mut buf, PrettyFormatter::new());
    value.serialize(&mut serializer)?;
    Ok(buf)
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    let mut serializer = JsonSerializer::new(&mut buf, CompactFormatter);
    value.serialize(&mut serializer)?;
    String::from_utf8(buf).map_err(|err| JsonSerializationError::Other(err.to_string()))
}

pub fn to_pretty_string<T: Serialize>(value: &T) -> Result<String, JsonSerializationError> {
    let mut buf = Vec::<u8>::new();
    let mut serializer = JsonSerializer::new(&mut buf, PrettyFormatter::new());
    value.serialize(&mut serializer)?;
    String::from_utf8(buf).map_err(|err| JsonSerializationError::Other(err.to_string()))
}
