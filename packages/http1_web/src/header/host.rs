use std::{ops::Deref, str::FromStr};

use http1::{headers, uri::uri::Uri};

use crate::error_response::ErrorStatusCode;

use super::FromHeaders;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host(pub Uri);

impl Host {
    pub fn into_inner(self) -> Uri {
        self.0
    }
}

impl Deref for Host {
    type Target = Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromHeaders for Host {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::HOST) {
            Some(value) => match Uri::from_str(value.as_str()) {
                Ok(uri) => Ok(Host(uri)),
                Err(err) => {
                    log::warn!("failed to parse `Host` uri: {err}");
                    Err(ErrorStatusCode::BadRequest)
                }
            },
            None => {
                log::warn!("`Host` header not found");
                Err(ErrorStatusCode::BadRequest)
            }
        }
    }
}
