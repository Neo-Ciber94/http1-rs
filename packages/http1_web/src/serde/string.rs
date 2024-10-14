use std::{fmt::Display, str::FromStr};

use crate::serde::visitor::Visitor;

use super::{
    de::{Deserializer, Error},
    impossible::Impossible,
    ser::Serializer,
    visitor::SeqAccess,
};

#[derive(Debug)]
pub enum DeserializeFromStr {
    Str(String),
    List(Vec<String>),
}

impl Deserializer for DeserializeFromStr {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `any`",
        ))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `unit`",
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = bool::from_str(&x).map_err(Error::error)?;
                visitor.visit_bool(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `bool`",
            )),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = u8::from_str(&x).map_err(Error::error)?;
                visitor.visit_u8(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `u8`",
            )),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = u16::from_str(&x).map_err(Error::error)?;
                visitor.visit_u16(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `u16`",
            )),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = u32::from_str(&x).map_err(Error::error)?;
                visitor.visit_u32(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `u32`",
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = u64::from_str(&x).map_err(Error::error)?;
                visitor.visit_u64(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `u64`",
            )),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = u128::from_str(&x).map_err(Error::error)?;
                visitor.visit_u128(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `u128`",
            )),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = i8::from_str(&x).map_err(Error::error)?;
                visitor.visit_i8(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `i8`",
            )),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = i16::from_str(&x).map_err(Error::error)?;
                visitor.visit_i16(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `i16`",
            )),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = i32::from_str(&x).map_err(Error::error)?;
                visitor.visit_i32(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `i32`",
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = i64::from_str(&x).map_err(Error::error)?;
                visitor.visit_i64(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `i64`",
            )),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = i128::from_str(&x).map_err(Error::error)?;
                visitor.visit_i128(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `i128`",
            )),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = f32::from_str(&x).map_err(Error::error)?;
                visitor.visit_f32(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `f32`",
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(x) => {
                let value = f64::from_str(&x).map_err(Error::error)?;
                visitor.visit_f64(value)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `f64`",
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(mut s) => {
                if s.is_empty() {
                    return Err(crate::serde::de::Error::custom(
                        "expected char but was empty",
                    ));
                }

                if s.len() != 1 {
                    return Err(crate::serde::de::Error::custom(format!(
                        "cannot deserialize `{s}` to char"
                    )));
                }

                let char = s.pop().expect("unable to get char");
                visitor.visit_char(char)
            }
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `char`",
            )),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(s) => visitor.visit_string(s),
            DeserializeFromStr::List(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize list to `string`",
            )),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        match self {
            DeserializeFromStr::Str(_) => Err(crate::serde::de::Error::custom(
                "cannot deserialize str to `map`",
            )),
            DeserializeFromStr::List(vec) => visitor.visit_seq(FromStrSeqAccess(vec.into_iter())),
        }
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `map`",
        ))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `option`",
        ))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `bytes`",
        ))
    }
}

struct FromStrSeqAccess(std::vec::IntoIter<String>);
impl SeqAccess for FromStrSeqAccess {
    fn next_element<D: super::de::Deserialize>(&mut self) -> Result<Option<D>, Error> {
        match self.0.next() {
            Some(x) => {
                let value = D::deserialize(DeserializeFromStr::Str(x))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

pub struct DeserializeOnlyString(pub String);

impl Deserializer for DeserializeOnlyString {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `any`"))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `unit`"))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `bool`"))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `u8`"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `u16`"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `u32`"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `u64`"))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `u128`"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `i8`"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `i16`"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `i32`"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `i64`"))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `i128`"))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `f32`"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `f64`"))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let mut s = self.0;

        if s.is_empty() {
            return Err(super::de::Error::custom("expected char but was empty"));
        }

        if s.len() != 1 {
            return Err(super::de::Error::custom(format!(
                "cannot deserialize `{s}` to char"
            )));
        }

        let char = s.pop().expect("unable to get char");
        visitor.visit_char(char)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom(
            "cannot deserialize str to `sequence`",
        ))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom("cannot deserialize str to `map`"))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        Err(super::de::Error::custom(
            "cannot deserialize str to `option`",
        ))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        let bytes = self.0.into_bytes();
        visitor.visit_bytes(bytes)
    }
}

pub struct StringSerializer;

#[derive(Debug)]
pub struct StringSerializationError;
impl Display for StringSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to serialize to string")
    }
}

impl std::error::Error for StringSerializationError {}

impl Serializer for StringSerializer {
    type Ok = String;
    type Err = StringSerializationError;
    type Seq = Impossible<Self::Ok, Self::Err>;
    type Map = Impossible<Self::Ok, Self::Err>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Err> {
        Ok(value.to_owned())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Err(StringSerializationError)
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Err(StringSerializationError)
    }
}
