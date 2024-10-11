use std::str::FromStr;

use http1::{
    body::http_body::HttpBody,
    error::BoxError,
    headers,
    status::StatusCode,
    uri::{
        convert::{self, InvalidUriComponent},
        path_query::PathAndQuery,
        uri::InvalidUri,
    },
};

use crate::{
    from_request::FromRequest, into_response::IntoResponse, query::QueryDeserializer,
    serde::de::Deserialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form<T>(T);

impl<T> Form<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

const WWW_FORM_URLENCODED: &str = "application/x-www-form-urlencoded";

#[doc(hidden)]
pub enum RejectFormError {
    NotContentType,
    InvalidContentType(String),
    FailedReadForm(BoxError),
    Utf8Error(BoxError),
    InvalidUriComponent(InvalidUriComponent),
    InvalidUri(InvalidUri),
    DeserializationError(BoxError),
}
impl IntoResponse for RejectFormError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match self {
            RejectFormError::NotContentType => {
                eprintln!("No content type: `{WWW_FORM_URLENCODED}`")
            }
            RejectFormError::InvalidContentType(s) => {
                eprintln!("Invalid content type `{s}` expected `{WWW_FORM_URLENCODED}`")
            }
            RejectFormError::FailedReadForm(error) => eprintln!("Failed to read form: {error}"),
            RejectFormError::Utf8Error(error) => eprintln!("Failed to read uf8 form {error}"),
            RejectFormError::InvalidUriComponent(_) => eprintln!("Failed to decode form"),
            RejectFormError::InvalidUri(invalid_uri) => {
                eprintln!("Failed to decode uri from form: {invalid_uri}")
            }
            RejectFormError::DeserializationError(error) => {
                eprintln!("Failed to deserialize form: {error}")
            }
        }

        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl<T: Deserialize> FromRequest for Form<T> {
    type Rejection = RejectFormError;

    fn from_request(
        mut req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        let content_type = headers
            .get(headers::CONTENT_TYPE)
            .ok_or(RejectFormError::NotContentType)?;

        match content_type.as_str().split(";").next() {
            Some(mime) => {
                if mime == WWW_FORM_URLENCODED {
                    return Err(RejectFormError::InvalidContentType(mime.to_owned()));
                }

                let bytes = req
                    .body_mut()
                    .read_all_bytes()
                    .map_err(|e| RejectFormError::FailedReadForm(e.into()))?;

                let s =
                    String::from_utf8(bytes).map_err(|e| RejectFormError::Utf8Error(e.into()))?;

                let q = convert::decode_uri_component(s)
                    .map_err(RejectFormError::InvalidUriComponent)?;

                let path_and_query =
                    PathAndQuery::from_str(&q).map_err(RejectFormError::InvalidUri)?;

                let query_map = path_and_query.query_map();
                T::deserialize(QueryDeserializer(query_map))
                    .map(Form)
                    .map_err(|e| RejectFormError::DeserializationError(e.into()))
            }
            None => Err(RejectFormError::NotContentType),
        }
    }
}
