use super::de::{Deserialize, Deserializer, Error};

pub trait Visitor: Sized {
    type Value;

    fn expected(&self) -> &'static str;

    fn visit_unit(self) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Unit, self))
    }

    fn visit_bool(self, value: bool) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Bool(value), self))
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
        Err(Error::mismatch(
            super::de::Unexpected::Unsigned(value),
            self,
        ))
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
        Err(Error::mismatch(super::de::Unexpected::Signed(value), self))
    }

    fn visit_f32(self, value: f32) -> Result<Self::Value, Error> {
        self.visit_f64(value.into())
    }

    fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Float(value), self))
    }

    fn visit_char(self, value: char) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Char(value), self))
    }

    fn visit_string(self, value: String) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Str(value), self))
    }

    fn visit_none(self) -> Result<Self::Value, Error> {
        Err(Error::mismatch(super::de::Unexpected::Option, self))
    }

    fn visit_some<D: Deserializer>(self, deserializer: D) -> Result<Self::Value, Error> {
        let _ = deserializer;
        Err(Error::mismatch(super::de::Unexpected::Option, self))
    }

    fn visit_seq<Seq: SeqAccess>(self, seq: Seq) -> Result<Self::Value, Error> {
        let _ = seq;
        Err(Error::mismatch(super::de::Unexpected::Seq, self))
    }

    fn visit_bytes_buf(self, bytes: Vec<u8>) -> Result<Self::Value, Error> {
        let _ = bytes;
        Err(Error::mismatch(super::de::Unexpected::Bytes, self))
    }

    fn visit_map<Map: MapAccess>(self, map: Map) -> Result<Self::Value, Error> {
        let _ = map;
        Err(Error::mismatch(super::de::Unexpected::Map, self))
    }

    fn visit_bytes_seq<B: BytesAccess>(self, bytes: B) -> Result<Self::Value, Error> {
        let _ = bytes;
        Err(Error::mismatch(super::de::Unexpected::Bytes, self))
    }
}

pub trait SeqAccess {
    fn next_element<D: Deserialize>(&mut self) -> Result<Option<D>, Error>;
}

pub trait MapAccess {
    fn next_entry<K: Deserialize, V: Deserialize>(&mut self) -> Result<Option<(K, V)>, Error> {
        let key = self.next_key::<K>()?;
        let value = self.next_value::<V>()?;

        match (key, value) {
            (Some(k), Some(v)) => Ok(Some((k, v))),
            _ => Ok(None),
        }
    }

    fn next_key<K: Deserialize>(&mut self) -> Result<Option<K>, Error>;
    fn next_value<V: Deserialize>(&mut self) -> Result<Option<V>, Error>;
}

pub trait BytesAccess {
    fn next_bytes<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), Error>;
}
