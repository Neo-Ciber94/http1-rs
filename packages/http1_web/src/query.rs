use std::fmt::Display;

use http1::{
    status::StatusCode,
    uri::path_query::{QueryMap, QueryValue},
};

use crate::{from_request::FromRequest, IntoResponse};
use serde::{
    de::{Deserialize, Deserializer, Error},
    string::{DeserializeFromStr, DeserializeOnlyString},
    visitor::MapAccess,
};

/// Represents the query params in a request.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Query<T>(pub T);

impl<T> Query<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub struct QueryDeserializer(pub QueryMap);

impl Deserializer for QueryDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `any`"))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `unit`"))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `bool`"))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `u8`"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `u16`"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `u32`"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `u64`"))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `u128`"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `i8`"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `i16`"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `i32`"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `i64`"))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `i128`"))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `f32`"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `f64`"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `char`"))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        let s = self.0.to_string();
        visitor.visit_string(s)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `seq`"))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_map(QueryMapAccess {
            iter: self.0.into_iter(),
            value: None,
        })
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `option`"))
    }

    fn deserialize_bytes_buf<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `bytes`"))
    }

    fn deserialize_bytes_seq<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(Error::other("cannot deserialize query params to `bytes`"))
    }
}

struct QueryMapAccess<I> {
    iter: I,
    value: Option<QueryValue>,
}

impl<I: Iterator<Item = (String, QueryValue)>> MapAccess for QueryMapAccess<I> {
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
            Some(x) => match x {
                QueryValue::One(val) => {
                    let value = V::deserialize(DeserializeFromStr::Str(val))?;
                    Ok(Some(value))
                }
                QueryValue::List(vec) => {
                    let value = V::deserialize(DeserializeFromStr::List(vec))?;
                    Ok(Some(value))
                }
            },
            None => Ok(None),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct InvalidQueryError;

impl Display for InvalidQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse request query params")
    }
}

impl IntoResponse for InvalidQueryError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl<T: Deserialize> FromRequest for Query<T> {
    type Rejection = InvalidQueryError;

    fn from_request(
        req: &http1::request::Request<()>,
        _payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        let query_map = req.uri().path_and_query().query_map();
        let deserializer = QueryDeserializer(query_map);
        T::deserialize(deserializer).map(Query).map_err(|err| {
            log::error!("Failed to deserialize query: {err}");
            InvalidQueryError
        })
    }
}
