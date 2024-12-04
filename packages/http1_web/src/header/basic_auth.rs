use http1::{body::Body, common::base64, headers, response::Response, status::StatusCode};

use crate::IntoResponse;

use super::FromHeaders;

/// Extracts and parse a `basic auth` header from the `Authentication` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[doc(hidden)]
pub struct Unauthorized;

impl IntoResponse for Unauthorized {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        Response::new(StatusCode::UNAUTHORIZED, Body::empty())
    }
}

impl FromHeaders for BasicAuth {
    type Rejection = Unauthorized;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::AUTHORIZATION) {
            Some(header_value) => {
                let mut header_str = header_value.as_str();

                if !header_str.starts_with("Basic ") {
                    return Err(Unauthorized);
                }

                header_str = &header_str["Basic ".len()..];

                let basic_auth_str = base64::decode(header_str).map_err(|_| Unauthorized)?;
                let (username, password) = basic_auth_str
                    .split_once(":")
                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
                    .ok_or_else(|| Unauthorized)?;

                Ok(BasicAuth { username, password })
            }
            None => Err(Unauthorized),
        }
    }
}
