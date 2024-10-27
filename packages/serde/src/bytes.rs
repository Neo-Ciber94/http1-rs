use crate::{
    de::{Deserialize, Deserializer},
    visitor::{BytesAccess, SeqAccess, Visitor},
};

pub struct BytesBufferDeserializer(pub Vec<u8>);

impl Deserializer for BytesBufferDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `any`",
        ))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `unit`",
        ))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `bool`",
        ))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `u8`",
        ))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `u16`",
        ))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `u32`",
        ))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `u64`",
        ))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `u128`",
        ))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `i8`",
        ))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `i16`",
        ))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `i32`",
        ))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `i64`",
        ))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `i128`",
        ))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `f32`",
        ))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `f64`",
        ))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `char`",
        ))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        let s = String::from_utf8_lossy(&self.0);
        visitor.visit_string(s.into_owned())
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `sequence`",
        ))
    }

    fn deserialize_bytes_buf<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_bytes_buf(self.0)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        let seq = BytesSeqAccess(self.0.into_iter());
        visitor.visit_seq(seq)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize bytes to `option`",
        ))
    }

    fn deserialize_bytes_seq<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        struct ByteBufferAccess(Vec<u8>);

        impl BytesAccess for ByteBufferAccess {
            fn next_bytes<W: std::io::Write>(
                &mut self,
                writer: &mut W,
            ) -> Result<(), super::de::Error> {
                writer
                    .write_all(self.0.as_slice())
                    .map_err(super::de::Error::error)
            }
        }

        visitor.visit_bytes_seq(ByteBufferAccess(self.0))
    }
}

struct ByteDeserializer(pub u8);
impl Deserializer for ByteDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `any`",
        ))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `unit`",
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_bool(self.0 != 0)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_u8(self.0)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_u16(self.0 as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_u32(self.0 as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_u64(self.0 as u64)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_u128(self.0 as u128)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_i8(self.0 as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_i16(self.0 as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_i32(self.0 as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_i64(self.0 as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_i128(self.0 as i128)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_f32(self.0 as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_f64(self.0 as f64)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let c = char::from_u32(self.0 as u32).ok_or_else(|| {
            super::de::Error::error(format!("failed to convert `{}` from char", self.0))
        })?;

        visitor.visit_char(c)
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `string`",
        ))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `seq`",
        ))
    }

    fn deserialize_bytes_buf<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let b = std::slice::from_ref(&self.0).to_vec();
        visitor.visit_bytes_buf(b)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `map`",
        ))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::de::Error::error(
            "failed to deserialize byte to `option`",
        ))
    }

    fn deserialize_bytes_seq<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        struct SingleByteAccess(Option<u8>);
        impl BytesAccess for SingleByteAccess {
            fn next_bytes<W: std::io::Write>(
                &mut self,
                writer: &mut W,
            ) -> Result<(), super::de::Error> {
                match self.0.take() {
                    Some(b) => {
                        let buf = std::slice::from_ref(&b);
                        writer.write_all(buf).map_err(super::de::Error::error)
                    }
                    None => Err(super::de::Error::error("no bytes to write")),
                }
            }
        }

        visitor.visit_bytes_seq(SingleByteAccess(Some(self.0)))
    }
}

struct BytesSeqAccess(std::vec::IntoIter<u8>);
impl SeqAccess for BytesSeqAccess {
    fn next_element<D: Deserialize>(&mut self) -> Result<Option<D>, crate::de::Error> {
        match self.0.next() {
            Some(byte) => {
                let v = D::deserialize(ByteDeserializer(byte))?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }
}
