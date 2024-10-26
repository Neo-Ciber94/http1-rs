use std::ops::Deref;

use http1::headers;

use crate::error_response::ErrorStatusCode;

use super::FromHeaders;

/// Represents the `If-Match` request header: [`https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Match`].
#[derive(Debug, Clone)]
pub struct IfMatch(Vec<String>);

impl IfMatch {
    pub fn contains(&self, value: &str) -> bool {
        self.0.iter().any(|x| x == value)
    }

    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl Deref for IfMatch {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl FromHeaders for IfMatch {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::IF_MATCH) {
            Some(value) => {
                let etags = value
                    .as_str()
                    .split(",")
                    .map(|x| x.trim())
                    .map(|x| x.to_owned())
                    .collect();
                Ok(IfMatch(etags))
            }
            None => {
                log::warn!("`If-Match` header was not found");
                Err(ErrorStatusCode::BadRequest)
            }
        }
    }
}
