use std::str::FromStr;

use crate::serde::visitor::Visitor;

use super::de::{Deserializer, Error};

#[derive(Debug)]
pub struct DeserializeFromStr(pub String);

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
        let value = bool::from_str(&self.0).map_err(Error::error)?; // Assuming Error::error is defined
        visitor.visit_bool(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = u8::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = u16::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = u32::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = u64::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_u64(value)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = u128::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_u128(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = i8::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = i16::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = i32::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = i64::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = i128::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_i128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = f32::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let value = f64::from_str(&self.0).map_err(Error::error)?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, super::de::Error>
    where
        V: Visitor,
    {
        let mut s = self.0;

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
        Err(crate::serde::de::Error::custom(
            "cannot deserialize str to `sequence`",
        ))
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
}
