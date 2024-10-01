pub struct DeserializeError;

pub trait Deserializer {
    fn read_u8(&self) -> Result<u8, DeserializeError>;
}

pub trait Deserialize {
    fn serialize<S: Deserializer>(&mut self, serializer: &S);
}
