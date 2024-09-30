use std::{borrow::Cow, convert::Infallible};

use http1::{
    body::Body,
    common::any_map::AnyMap,
    error::BoxError,
    headers::{self, Headers},
    response::Response,
    status::StatusCode,
};

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

impl<'a> IntoResponse for Cow<'a, str> {
    fn into_response(self) -> Response<Body> {
        self.as_ref().into_response()
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

impl<'a> IntoResponse for Cow<'a, [u8]> {
    fn into_response(self) -> Response<Body> {
        self.as_ref().into_response()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response<Body> {
        Response::new(self, Body::empty())
    }
}

impl IntoResponse for Headers {
    fn into_response(self) -> Response<Body> {
        let mut res = Response::new(StatusCode::OK, Body::empty());
        res.headers_mut().extend(self);
        res
    }
}

impl<T: IntoResponse> IntoResponse for Option<T> {
    fn into_response(self) -> Response<Body> {
        match self {
            Some(x) => x.into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        match self {
            Ok(x) => x.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl IntoResponse for std::io::Error {
    fn into_response(self) -> Response<Body> {
        match self.kind() {
            std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
            std::io::ErrorKind::PermissionDenied => StatusCode::UNAUTHORIZED.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

/// Represents segments of the response.
pub struct ResponseParts {
    response: Response<Body>,
}

impl ResponseParts {
    /// Returns the headers of the response.
    pub fn headers(&self) -> &Headers {
        self.response.headers()
    }

    /// Returns a mutable reference to the headers of the response.
    pub fn headers_mut(&mut self) -> &mut Headers {
        self.response.headers_mut()
    }

    /// Returns the extensions of the response.
    pub fn extensions(&self) -> &AnyMap {
        self.response.extensions()
    }

    /// Returns a mutable reference to the extensions of the response.
    pub fn extensions_mut(&mut self) -> &mut AnyMap {
        self.response.extensions_mut()
    }
}

/// Provide an interface to update the existing response.
pub trait IntoResponseParts {
    type Err: Into<BoxError>;

    /// Returns the new parts of the response.
    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Err>;
}

impl IntoResponseParts for Headers {
    type Err = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Err> {
        res.headers_mut().extend(self);
        Ok(res)
    }
}

impl IntoResponseParts for AnyMap {
    type Err = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Err> {
        res.extensions_mut().extend(self);
        Ok(res)
    }
}

impl<T> IntoResponseParts for Option<T>
where
    T: IntoResponseParts,
{
    type Err = T::Err;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Err> {
        match self {
            Some(x) => {
                let res = x.into_response_parts(res)?;
                Ok(res)
            }
            None => Ok(res),
        }
    }
}

impl<T1, T2> IntoResponseParts for (T1, T2)
where
    T1: IntoResponseParts,
    T2: IntoResponseParts,
    T1::Err: std::error::Error + Send + Sync + 'static,
    T2::Err: std::error::Error + Send + Sync + 'static,
{
    type Err = BoxError;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Err> {
        let (t1, t2) = self;
        let res = t1.into_response_parts(res)?;
        let res = t2.into_response_parts(res)?;
        Ok(res)
    }
}

impl<R, T1> IntoResponse for (R, T1)
where
    R: IntoResponse,
    T1: IntoResponseParts,
    T1::Err: std::error::Error + Send + Sync + 'static,
{
    fn into_response(self) -> Response<Body> {
        let (t1, t2) = self;
        let response = t1.into_response();
        let res = ResponseParts { response };

        match t2.into_response_parts(res) {
            Ok(res) => res.response,
            Err(err) => {
                eprintln!(
                    "Failed to convert {} into response parts: {err}",
                    std::any::type_name::<T1>()
                );
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
