use std::fmt::Display;

use http1::{
    body::http_body::HttpBody,
    common::map::OrderedMap,
    error::BoxError,
    headers,
    response::Response,
    status::StatusCode,
    uri::{
        url_encoding::{self, InvalidUriComponent},
        path_query::{QueryMap, QueryValue},
        uri::InvalidUri,
    },
};

use crate::{
    from_request::FromRequest,
    into_response::IntoResponse,
    query::QueryDeserializer,
    serde::{
        de::Deserialize,
        impossible::Impossible,
        ser::{MapSerializer, Serialize, Serializer},
        string::StringSerializer,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form<T>(pub T);

impl<T> Form<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

const WWW_FORM_URLENCODED: &str = "application/x-www-form-urlencoded";

#[doc(hidden)]
#[derive(Debug)]
pub enum RejectFormError {
    NoContentType,
    InvalidContentType(String),
    FailedReadForm(BoxError),
    Utf8Error(BoxError),
    InvalidUriComponent(InvalidUriComponent),
    InvalidUri(InvalidUri),
    DeserializationError(BoxError),
}

impl Display for RejectFormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RejectFormError::NoContentType => {
                write!(f, "No content type: `{WWW_FORM_URLENCODED}`")
            }
            RejectFormError::InvalidContentType(s) => {
                write!(
                    f,
                    "Invalid content type `{s}` expected `{WWW_FORM_URLENCODED}`"
                )
            }
            RejectFormError::FailedReadForm(error) => write!(f, "Failed to read form: {error}"),
            RejectFormError::Utf8Error(error) => write!(f, "Failed to read uf8 form {error}"),
            RejectFormError::InvalidUriComponent(_) => write!(f, "Failed to decode form"),
            RejectFormError::InvalidUri(invalid_uri) => {
                write!(f, "Failed to decode uri from form: {invalid_uri}")
            }
            RejectFormError::DeserializationError(error) => {
                write!(f, "Failed to deserialize form: {error}")
            }
        }
    }
}

impl IntoResponse for RejectFormError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl std::error::Error for RejectFormError {}

impl<T: Deserialize> FromRequest for Form<T> {
    type Rejection = RejectFormError;

    fn from_request(
        mut req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        let content_type = headers
            .get(headers::CONTENT_TYPE)
            .ok_or(RejectFormError::NoContentType)?;

        match content_type.as_str().split(";").next() {
            Some(mime) => {
                if mime != WWW_FORM_URLENCODED {
                    return Err(RejectFormError::InvalidContentType(mime.to_owned()));
                }

                let bytes = req
                    .body_mut()
                    .read_all_bytes()
                    .map_err(|e| RejectFormError::FailedReadForm(e.into()))?;

                let s =
                    String::from_utf8(bytes).map_err(|e| RejectFormError::Utf8Error(e.into()))?;

                let q = url_encoding::decode(&s)
                    .map_err(RejectFormError::InvalidUriComponent)?;

                let query_map = QueryMap::from_query_str(&q);

                T::deserialize(QueryDeserializer(query_map))
                    .map(Form)
                    .map_err(|e| RejectFormError::DeserializationError(e.into()))
            }
            None => Err(RejectFormError::NoContentType),
        }
    }
}

#[derive(Debug)]
struct SerializeFormError;

impl Display for SerializeFormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to serialize form")
    }
}

impl std::error::Error for SerializeFormError {}

struct FormSerializer;
impl Serializer for FormSerializer {
    type Ok = OrderedMap<String, QueryValue>;
    type Err = SerializeFormError;
    type Bytes = Impossible<Self::Ok, Self::Err>;
    type Seq = Impossible<Self::Ok, Self::Err>;
    type Map = Impossible<Self::Ok, Self::Err>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Err(SerializeFormError)
    }

    fn serialize_byte_seq(self) -> Result<Self::Bytes, Self::Err> {
        Err(SerializeFormError)
    }
}

struct FormMapSerializer(OrderedMap<String, QueryValue>);
impl MapSerializer for FormMapSerializer {
    type Ok = OrderedMap<String, QueryValue>;
    type Err = SerializeFormError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        let k = key
            .serialize(StringSerializer)
            .map_err(|_| SerializeFormError)?;

        let v = crate::serde::json::to_string(value).map_err(|_| SerializeFormError)?;

        if self.0.contains_key(&k) {
            let query_value = self.0.get_mut(&k).unwrap();
            match query_value {
                QueryValue::One(x) => {
                    let s = std::mem::take(x);
                    let _ = std::mem::replace(query_value, QueryValue::List(vec![s, v]));
                }
                QueryValue::List(values) => values.push(v),
            }
        } else {
            self.0.insert(k, QueryValue::One(v));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

impl<T: Serialize> IntoResponse for Form<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match self.0.serialize(FormSerializer) {
            Ok(map) => {
                let s = QueryMap::new(map).to_string();
                let body = s.into();
                Response::builder()
                    .append_header(headers::CONTENT_TYPE, WWW_FORM_URLENCODED)
                    .body(body)
            }
            Err(err) => {
                log::error!("Failed to serialize form: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
