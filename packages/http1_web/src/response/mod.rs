mod error_response;
mod http_response;
mod into_response;

use std::fmt::Display;

use http1::{body::Body, headers::HeaderValue, response::Response, status::StatusCode};
pub use {error_response::*, http_response::*, into_response::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WWWAuthenticate {
    realm: String,
    charset: Option<String>,
}

impl WWWAuthenticate {
    pub fn new(realm: impl Into<String>) -> Self {
        WWWAuthenticate {
            realm: realm.into(),
            charset: None,
        }
    }

    pub fn with_charset(realm: impl Into<String>, charset: impl Into<String>) -> Self {
        WWWAuthenticate {
            realm: realm.into(),
            charset: Some(charset.into()),
        }
    }

    pub fn realm(&self) -> &str {
        &self.realm
    }

    pub fn charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }
}

impl Display for WWWAuthenticate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Basic realm=\"{}\"", self.realm)?;

        if let Some(charset) = &self.charset {
            write!(f, ", charset=\"{charset}\"")?;
        }

        Ok(())
    }
}

impl IntoResponse for WWWAuthenticate {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let header_value = HeaderValue::from_string(self.to_string());

        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .insert_header(http1::headers::WWW_AUTHENTICATE, header_value)
            .body(Body::empty())
    }
}
