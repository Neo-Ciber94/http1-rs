use crate::{de::Deserialize, visitor::Visitor};

/// Allow to ignore any value for deserialization.
pub struct Ignore;

impl Visitor for Ignore {
    type Value = Ignore;

    fn expected(&self) -> &'static str {
        "ignore anything"
    }

    fn visit_unit(self) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_bool(self, _value: bool) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_u128(self, _value: u128) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_i128(self, _value: i128) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_f64(self, _value: f64) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_char(self, _value: char) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_string(self, _value: String) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_none(self) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_some<D: crate::de::Deserializer>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, crate::de::Error> {
        deserializer.deserialize_any(Ignore)
    }

    fn visit_seq<Seq: crate::visitor::SeqAccess>(
        self,
        _seq: Seq,
    ) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_bytes_buf(self, _bytes: Vec<u8>) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_map<Map: crate::visitor::MapAccess>(
        self,
        _map: Map,
    ) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }

    fn visit_bytes_seq<B: crate::visitor::BytesAccess>(
        self,
        _bytes: B,
    ) -> Result<Self::Value, crate::de::Error> {
        Ok(Ignore)
    }
}

impl Deserialize for Ignore {
    fn deserialize<D: crate::de::Deserializer>(deserializer: D) -> Result<Self, crate::de::Error> {
        deserializer.deserialize_any(Ignore)
    }
}
