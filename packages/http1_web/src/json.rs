use std::fmt::Display;

use http1::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
    headers::CONTENT_TYPE,
    response::Response,
    status::StatusCode,
};

use crate::{from_request::FromRequest, mime::Mime, IntoResponse};

use serde::{self, de::Deserialize, ser::Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);

#[doc(hidden)]
#[derive(Debug)]
pub enum InvalidJsonError {
    NoBody,
    Other(BoxError),
}

impl Display for InvalidJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidJsonError::NoBody => write!(f, "request body was already taken"),
            InvalidJsonError::Other(error) => write!(f, "failed to parse json: {error:?}"),
        }
    }
}

impl std::error::Error for InvalidJsonError {}

impl IntoResponse for InvalidJsonError {
    fn into_response(self) -> Response<Body> {
        log::error!("{self}");
        match self {
            InvalidJsonError::NoBody => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            InvalidJsonError::Other(..) => StatusCode::UNPROCESSABLE_CONTENT.into_response(),
        }
    }
}

impl<T: Deserialize> FromRequest for Json<T> {
    type Rejection = InvalidJsonError;

    fn from_request(
        _req: &http1::request::Request<()>,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        if payload.is_empty() {
            return Err(InvalidJsonError::NoBody);
        }

        let bytes = payload.read_all_bytes().map_err(InvalidJsonError::Other)?;
        let value =
            serde::json::from_bytes::<T>(bytes).map_err(|e| InvalidJsonError::Other(e.into()))?;
        Ok(Json(value))
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match serde::json::to_string(&self.0) {
            Ok(x) => Response::builder()
                .insert_header(CONTENT_TYPE, Mime::APPLICATION_JSON_UTF8)
                .body(x.into()),
            Err(err) => {
                log::error!("Failed to create JSON response: {err}");
                Response::new(StatusCode::UNPROCESSABLE_CONTENT, Body::empty())
            }
        }
    }
}
