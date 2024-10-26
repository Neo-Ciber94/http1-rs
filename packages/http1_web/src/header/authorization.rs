use std::ops::Deref;

use http1::headers;

use crate::error_response::ErrorStatusCode;

use super::FromHeaders;

/// Represents the `Authorization` request header: [`https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Authorization(pub String);

impl Authorization {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for Authorization {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromHeaders for Authorization {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::AUTHORIZATION) {
            Some(value) => Ok(Authorization(value.to_string())),
            None => {
                log::warn!("`Authorization` header not found");
                Err(ErrorStatusCode::Unauthorized)
            }
        }
    }
}
