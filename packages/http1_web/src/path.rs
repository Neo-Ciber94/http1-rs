use crate::{
    from_request::FromRequestRef,
    router::params::ParamsMap,
    serde::{
        de::{Deserialize, Deserializer},
        string::{DeserializeFromStr, DeserializeOnlyString},
        visitor::{MapAccess, SeqAccess},
    },
};
use std::str::FromStr;

/// Represents the path params in a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path<T>(pub T);

impl<T> Path<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Deserialize> FromRequestRef for Path<T> {
    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, http1::error::BoxError> {
        let params_map = req
            .extensions()
            .get::<ParamsMap>()
            .cloned()
            .ok_or_else(|| "failed to get params map")?;

        let value = T::deserialize(PathDeserializer(params_map))?;
        Ok(Path(value))
    }
}

pub struct PathDeserializer(ParamsMap);

impl Deserializer for PathDeserializer {
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;
        visitor.visit_string(s.to_owned())
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = bool::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_bool(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = u8::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = u16::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = u32::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = u64::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_u64(value)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = u128::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_u128(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = i8::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = i16::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = i32::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = i64::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = i128::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_i128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = f32::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        let value = f64::from_str(s).map_err(crate::serde::de::Error::error)?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

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

        let char = s.chars().next().expect("unable to get char");
        visitor.visit_char(char)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| crate::serde::de::Error::custom("cannot get first param of the path"))?;

        visitor.visit_string(s.to_owned())
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        return Err(crate::serde::de::Error::custom(format!(
            "cannot deserialize `option` to char"
        )));
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        visitor.visit_seq(ParamsSeqAccess(self.0.into_iter()))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        visitor.visit_map(ParamsMapAccess {
            iter: self.0.into_iter(),
            value: None,
        })
    }
}

struct ParamsSeqAccess<I>(I);

impl<I: Iterator<Item = (String, String)>> SeqAccess for ParamsSeqAccess<I> {
    fn next_element<D: crate::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<D>, crate::serde::de::Error> {
        match self.0.next() {
            Some((_, value)) => {
                let v = D::deserialize(DeserializeFromStr(value))?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }
}

struct ParamsMapAccess<I> {
    iter: I,
    value: Option<String>,
}

impl<I: Iterator<Item = (String, String)>> MapAccess for ParamsMapAccess<I> {
    fn next_key<K: crate::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<K>, crate::serde::de::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = K::deserialize(DeserializeOnlyString(k))?;
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    fn next_value<V: crate::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<V>, crate::serde::de::Error> {
        match self.value.take() {
            Some(x) => {
                let value = V::deserialize(DeserializeFromStr(x))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
