use super::de::{Deserialize, Error};

pub trait Visitor: Sized {
    type Value;

    fn visit_unit(self) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Unit))
    }

    fn visit_bool(self, value: bool) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Bool(value)))
    }

    fn visit_u8(self, value: u8) -> Result<Self::Value, Error> {
        self.visit_u128(value.into())
    }

    fn visit_u16(self, value: u16) -> Result<Self::Value, Error> {
        self.visit_u128(value.into())
    }

    fn visit_u32(self, value: u32) -> Result<Self::Value, Error> {
        self.visit_u128(value.into())
    }

    fn visit_u64(self, value: u64) -> Result<Self::Value, Error> {
        self.visit_u128(value.into())
    }

    fn visit_u128(self, value: u128) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Unsigned(value)))
    }

    fn visit_i8(self, value: i8) -> Result<Self::Value, Error> {
        self.visit_i128(value.into())
    }

    fn visit_i16(self, value: i16) -> Result<Self::Value, Error> {
        self.visit_i128(value.into())
    }

    fn visit_i32(self, value: i32) -> Result<Self::Value, Error> {
        self.visit_i128(value.into())
    }

    fn visit_i64(self, value: i64) -> Result<Self::Value, Error> {
        self.visit_i128(value.into())
    }

    fn visit_i128(self, value: i128) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Signed(value)))
    }

    fn visit_f32(self, value: f32) -> Result<Self::Value, Error> {
        self.visit_f64(value.into())
    }

    fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
        return Err(Error::Unexpected(super::de::Unexpected::Float(value)));
    }

    fn visit_char(self, value: char) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Char(value)))
    }

    fn visit_string(self, value: String) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Str(value)))
    }

    fn visit_option<T>(self, _value: Option<T>) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Option))
    }

    fn visit_seq<Seq: SeqAccess>(self, _seq: Seq) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Seq))
    }

    fn visit_map<Map: MapAccess>(self, _map: Map) -> Result<Self::Value, Error> {
        Err(Error::Unexpected(super::de::Unexpected::Map))
    }
}

pub trait SeqAccess {
    fn next_element<T: Deserialize>(&mut self) -> Result<Option<T>, Error>;
}

pub trait MapAccess {
    fn next_entry<K: Deserialize, V: Deserialize>(&mut self) -> Result<Option<(K, V)>, Error>;
}
