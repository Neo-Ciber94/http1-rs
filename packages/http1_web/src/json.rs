use http1::{
    body::{http_body::HttpBody, Body},
    headers::CONTENT_TYPE,
    response::Response,
    status::StatusCode,
};

use crate::{
    common::{
        self,
        serde::{de::Deserialize, ser::Serialize},
    },
    from_request::FromRequest,
    into_response::IntoResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);

impl<T: Deserialize> FromRequest for Json<T> {
    fn from_request(
        mut req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, http1::error::BoxError> {
        let bytes = req.body_mut().read_all_bytes()?;
        let value = common::serde::json::from_bytes::<T>(bytes)?;
        Ok(Json(value))
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match common::serde::json::to_string(&self.0) {
            Ok(x) => Response::builder()
                .insert_header(CONTENT_TYPE, "application/json; charset=UTF-8")
                .build(x.into()),
            Err(err) => {
                eprintln!("Failed to create JSON response: {err}");
                Response::new(StatusCode::UNPROCESSABLE_CONTENT, Body::empty())
            }
        }
    }
}
