use super::FromHeaders;
use crate::ErrorStatusCode;
use http1::headers;
use std::ops::Deref;

/// Represents the raw `User-Agent` request header: [`https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawUserAgent(pub String);

impl RawUserAgent {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for RawUserAgent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromHeaders for RawUserAgent {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::USER_AGENT) {
            Some(value) => Ok(RawUserAgent(value.to_string())),
            None => {
                log::warn!("`User-Agent` header was not found");
                Err(ErrorStatusCode::BadRequest)
            }
        }
    }
}
