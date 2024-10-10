use http1::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
    headers::CONTENT_TYPE,
    response::Response,
    status::StatusCode,
};

use crate::{
    from_request::FromRequest,
    into_response::IntoResponse,
    serde::{self, de::Deserialize, ser::Serialize},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);

#[doc(hidden)]
pub struct InvalidJsonError(BoxError);
impl IntoResponse for InvalidJsonError {
    fn into_response(self) -> Response<Body> {
        eprintln!("Failed to parse json: {}", self.0);
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl<T: Deserialize> FromRequest for Json<T> {
    type Rejection = InvalidJsonError;

    fn from_request(
        mut req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let bytes = req
            .body_mut()
            .read_all_bytes()
            .map_err(|e| InvalidJsonError(e.into()))?;
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
                eprintln!("Failed to create JSON response: {err}");
                Response::new(StatusCode::UNPROCESSABLE_CONTENT, Body::empty())
            }
        }
    }
}
