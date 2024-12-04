use std::ops::{Deref, DerefMut};

use http1::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
    headers::{self, HeaderName, HeaderValue},
    response::Response,
    status::StatusCode,
};
use serde::ser::Serialize;

use crate::{cookies::Cookie, mime};

use super::IntoResponse;

/// A convenient `HttpResponse` builder for creating HTTP responses with
/// common status codes and custom headers.
#[derive(Debug)]
pub struct HttpResponse<B = Body>(Response<B>);

impl<B> HttpResponse<B> {
    /// Consumes the `HttpResponse` and returns the underlying `Response`.
    pub fn into_inner(self) -> Response<B> {
        self.0
    }
}

impl HttpResponse<()> {
    /// Creates a `HttpResponse` from a value.
    pub fn from_response<T: IntoResponse>(value: T) -> HttpResponse<Body> {
        let res = value.into_response();
        HttpResponse(res)
    }
}

impl<B> HttpResponse<B>
where
    B: HttpBody + Send + 'static,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    /// Creates a new `HttpResponse` with a specified status code and body.
    pub fn new(status: StatusCode, body: B) -> Self {
        HttpResponse(Response::new(status, body))
    }

    /// Sets the status code of the `HttpResponse`.
    pub fn status_code(mut self, status: StatusCode) -> Self {
        *self.0.status_mut() = status;
        self
    }

    /// Appends a header to the `HttpResponse`, adding another header if it
    /// already exists.
    pub fn append_header(mut self, name: HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.0.headers_mut().append(name, value.into());
        self
    }

    /// Inserts a header into the `HttpResponse`, replacing the existing one if
    /// it already exists.
    pub fn insert_header(mut self, name: HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.0.headers_mut().insert(name, value.into());
        self
    }

    /// Sets a cookie header in the `HttpResponse`.
    pub fn set_cookie(self, cookie: impl Into<Cookie>) -> Self {
        self.append_header(
            headers::SET_COOKIE,
            HeaderValue::from_string(cookie.into().to_string()),
        )
    }

    /// Maps the body of the `HttpResponse` to a different type.
    pub fn map_body<R: Into<Body>, F: FnOnce(B) -> R>(self, f: F) -> HttpResponse<R> {
        HttpResponse(self.0.map_body(f))
    }

    /// Map the body to `Body`.
    pub fn finish(self) -> HttpResponse<Body> {
        HttpResponse(self.0.map_body(Body::new))
    }
}

impl HttpResponse<()> {
    /// Creates a JSON `HttpResponse` from a serializable value.
    pub fn json<T: Serialize>(value: &T) -> Result<HttpResponse<String>, BoxError> {
        let json = serde::json::to_string(&value)?;
        let res = HttpResponse::ok(json)
            .append_header(headers::CONTENT_TYPE, mime::Mime::APPLICATION_JSON_UTF8);
        Ok(res)
    }

    /// Creates a `HttpResponse` with a 301 Moved Permanently status and `Location` header.
    pub fn moved_permanently(location: impl Into<String>) -> Self {
        HttpResponse::new(StatusCode::MOVED_PERMANENTLY, ())
            .insert_header(headers::LOCATION, HeaderValue::from_string(location.into()))
    }

    /// Creates a `HttpResponse` with a 302 Found status and `Location` header.
    pub fn found(location: impl Into<String>) -> Self {
        HttpResponse::new(StatusCode::FOUND, ())
            .insert_header(headers::LOCATION, HeaderValue::from_string(location.into()))
    }

    /// Creates a `HttpResponse` with a 303 See Other status and `Location` header.
    pub fn see_other(location: impl Into<String>) -> Self {
        HttpResponse::new(StatusCode::SEE_OTHER, ())
            .insert_header(headers::LOCATION, HeaderValue::from_string(location.into()))
    }

    /// Creates a `HttpResponse` with a 304 Not Modified status.
    pub fn not_modified() -> Self {
        HttpResponse::new(StatusCode::NOT_MODIFIED, ())
    }

    /// Creates a `HttpResponse` with a 307 Temporary Redirect status and `Location` header.
    pub fn temporary_redirect(location: impl Into<String>) -> Self {
        HttpResponse::new(StatusCode::TEMPORARY_REDIRECT, ())
            .insert_header(headers::LOCATION, HeaderValue::from_string(location.into()))
    }

    /// Creates a `HttpResponse` with a 308 Permanent Redirect status and `Location` header.
    pub fn permanent_redirect(location: impl Into<String>) -> Self {
        HttpResponse::new(StatusCode::PERMANENT_REDIRECT, ())
            .insert_header(headers::LOCATION, HeaderValue::from_string(location.into()))
    }
}

impl<B> HttpResponse<B>
where
    B: HttpBody + Send + 'static,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    /// Creates an `HttpResponse` with a 200 OK status and a specified body.
    pub fn ok(body: B) -> Self {
        Self::new(StatusCode::OK, body)
    }

    /// Creates an `HttpResponse` with a 201 Created status and a specified body.
    pub fn created(body: B) -> Self {
        Self::new(StatusCode::CREATED, body)
    }

    /// Creates an `HttpResponse` with a 400 Bad Request status and a specified body.
    pub fn bad_request(body: B) -> Self {
        Self::new(StatusCode::BAD_REQUEST, body)
    }

    /// Creates an `HttpResponse` with a 401 Unauthorized status and a specified body.
    pub fn unauthorized(body: B) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, body)
    }

    /// Creates an `HttpResponse` with a 403 Forbidden status and a specified body.
    pub fn forbidden(body: B) -> Self {
        Self::new(StatusCode::FORBIDDEN, body)
    }

    /// Creates an `HttpResponse` with a 404 Not Found status and a specified body.
    pub fn not_found(body: B) -> Self {
        Self::new(StatusCode::NOT_FOUND, body)
    }

    /// Creates an `HttpResponse` with a 500 Internal Server Error status and a specified body.
    pub fn internal_server_error(body: B) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, body)
    }
}

impl<B> Deref for HttpResponse<B> {
    type Target = Response<B>;

    /// Dereferences the `HttpResponse` to access the inner `Response`.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> DerefMut for HttpResponse<B> {
    /// Dereferences the `HttpResponse` mutably to access the inner `Response`.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<B> IntoResponse for HttpResponse<B>
where
    B: HttpBody + Send + 'static,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    /// Converts the `HttpResponse` into a generic `Response<Body>`.
    fn into_response(self) -> Response<Body> {
        self.0.map_body(Body::new)
    }
}
