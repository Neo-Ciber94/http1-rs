use std::borrow::Cow;

use http1::{body::Body, headers, response::Response};

/**
 * Represents an object that can be converted to a response.
 */
pub trait IntoResponse {
    /**
     * Converts this object into a response.
     */
    fn into_response(self) -> Response<Body>;
}

impl<T: Into<Body>> IntoResponse for Response<T> {
    fn into_response(self) -> Response<Body> {
        self.map_body(|x| x.into())
    }
}

impl<'a> IntoResponse for &'a str {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .insert_header(headers::CONTENT_TYPE, "text/plain")
            .build(self.into())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Body> {
        self.as_str().into_response()
    }
}

impl<'a> IntoResponse for Cow<'a, String> {
    fn into_response(self) -> Response<Body> {
        self.as_str().into_response()
    }
}

impl<'a> IntoResponse for &'a [u8] {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .insert_header(headers::CONTENT_TYPE, "application/octet-stream")
            .build(self.into())
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response<Body> {
        self.as_slice().into_response()
    }
}

impl<'a> IntoResponse for Cow<'a, Vec<u8>> {
    fn into_response(self) -> Response<Body> {
        self.as_slice().into_response()
    }
}
