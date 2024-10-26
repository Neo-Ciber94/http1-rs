use std::{ops::Deref, str::FromStr};

use http1::headers;

use crate::{error_response::ErrorStatusCode, mime::Mime};

use super::FromHeaders;

/// Represents the `Accept` request header: [`https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept`].
pub struct Accept(pub Mime);

impl Accept {
    pub fn into_inner(self) -> Mime {
        self.0
    }
}

impl Deref for Accept {
    type Target = Mime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromHeaders for Accept {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::ACCEPT) {
            Some(value) => match Mime::from_str(value.as_str()) {
                Ok(mime) => Ok(Accept(mime)),
                Err(err) => {
                    log::warn!("Failed to parse `Accept` header: {err}");
                    Err(ErrorStatusCode::BadRequest)
                }
            },
            None => {
                log::warn!("`Accept` header not found");
                Err(ErrorStatusCode::BadRequest)
            }
        }
    }
}
