use http1::{error::BoxError, status::StatusCode};

use crate::{from_request::FromRequestRef, routing::params::ParamsMap, IntoResponse};
use serde::{
    de::{Deserialize, Deserializer},
    string::{DeserializeFromStr, DeserializeOnlyString},
    visitor::{MapAccess, SeqAccess},
};
use std::{fmt::Display, str::FromStr};

/// Represents the path params in a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path<T>(pub T);

impl<T> Path<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum PathRejectionError {
    NotParamsMap,
    DeserializationError(BoxError),
}

impl Display for PathRejectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathRejectionError::NotParamsMap => write!(f, "no params found"),
            PathRejectionError::DeserializationError(error) => {
                write!(f, "Failed to deserialize path: {error}")
            }
        }
    }
}

impl IntoResponse for PathRejectionError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        match self {
            PathRejectionError::NotParamsMap => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            PathRejectionError::DeserializationError(_) => {
                StatusCode::UNPROCESSABLE_CONTENT.into_response()
            }
        }
    }
}

impl std::error::Error for PathRejectionError {}

impl<T: Deserialize> FromRequestRef for Path<T> {
    type Rejection = PathRejectionError;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let params_map = req
            .extensions()
            .get::<ParamsMap>()
            .cloned()
            .ok_or(PathRejectionError::NotParamsMap)?;

        let value = T::deserialize(PathDeserializer(params_map))
            .map_err(|err| PathRejectionError::DeserializationError(err.into()))?;

        Ok(Path(value))
    }
}

pub struct PathDeserializer(ParamsMap);

impl Deserializer for PathDeserializer {
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;
        visitor.visit_string(s.to_owned())
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = bool::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_bool(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = u8::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = u16::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = u32::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = u64::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_u64(value)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = u128::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_u128(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = i8::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = i16::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = i32::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = i64::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = i128::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_i128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = f32::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        let value = f64::from_str(s).map_err(serde::de::Error::error)?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        if s.is_empty() {
            return Err(serde::de::Error::error("expected char but was empty"));
        }

        if s.len() != 1 {
            return Err(serde::de::Error::error(format!(
                "cannot deserialize `{s}` to char"
            )));
        }

        let char = s.chars().next().expect("unable to get char");
        visitor.visit_char(char)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self
            .0
            .get_index(0)
            .ok_or_else(|| serde::de::Error::error("cannot get first param of the path"))?;

        visitor.visit_string(s.to_owned())
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(String::from(
            "cannot deserialize `path` to option"
        )))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_seq(ParamsSeqAccess(self.0.into_iter()))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_map(ParamsMapAccess {
            iter: self.0.into_iter(),
            value: None,
        })
    }

    fn deserialize_bytes_buf<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(String::from(
            "cannot deserialize `path` to bytes",
        )))
    }

    fn deserialize_bytes_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(String::from(
            "cannot deserialize `path` to bytes",
        )))
    }
}

struct ParamsSeqAccess<I>(I);

impl<I: Iterator<Item = (String, String)>> SeqAccess for ParamsSeqAccess<I> {
    fn next_element<D: serde::de::Deserialize>(&mut self) -> Result<Option<D>, serde::de::Error> {
        match self.0.next() {
            Some((_, value)) => {
                let v = D::deserialize(DeserializeFromStr::Str(value))?;
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
    fn next_key<K: serde::de::Deserialize>(&mut self) -> Result<Option<K>, serde::de::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = K::deserialize(DeserializeOnlyString(k))?;
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    fn next_value<V: serde::de::Deserialize>(&mut self) -> Result<Option<V>, serde::de::Error> {
        match self.value.take() {
            Some(x) => {
                let value = V::deserialize(DeserializeFromStr::Str(x))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
