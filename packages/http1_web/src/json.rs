use std::fmt::Display;

use http1::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
    headers::CONTENT_TYPE,
    response::Response,
    status::StatusCode,
};

use crate::{from_request::FromRequest, IntoResponse};

use serde::{self, de::Deserialize, ser::Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);

#[doc(hidden)]
#[derive(Debug)]
pub struct InvalidJsonError(BoxError);

impl Display for InvalidJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse json: {}", self.0)
    }
}

impl std::error::Error for InvalidJsonError {}

impl IntoResponse for InvalidJsonError {
    fn into_response(self) -> Response<Body> {
        log::error!("{self}");
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl<T: Deserialize> FromRequest for Json<T> {
    type Rejection = InvalidJsonError;

    fn from_request(
        mut req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let bytes = req.body_mut().read_all_bytes().map_err(InvalidJsonError)?;
        let value = serde::json::from_bytes::<T>(bytes).map_err(|e| InvalidJsonError(e.into()))?;
        Ok(Json(value))
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match serde::json::to_string(&self.0) {
            Ok(x) => Response::builder()
                .insert_header(CONTENT_TYPE, "application/json; charset=UTF-8")
                .body(x.into()),
            Err(err) => {
                log::error!("Failed to create JSON response: {err}");
                Response::new(StatusCode::UNPROCESSABLE_CONTENT, Body::empty())
            }
        }
    }
}
