use std::{fmt::Display, ops::Deref, str::FromStr};

use http1::{headers, uri::uri::Uri};

use crate::ErrorStatusCode;

use super::FromHeaders;

/// Represents the `Referer` request header: [`https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referer`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Referer(pub Uri);

impl Referer {
    pub fn into_inner(self) -> Uri {
        self.0
    }
}

impl Deref for Referer {
    type Target = Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Referer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromHeaders for Referer {
    type Rejection = ErrorStatusCode;

    fn from_headers(headers: &http1::headers::Headers) -> Result<Self, Self::Rejection> {
        match headers.get(headers::REFERER) {
            Some(value) => match Uri::from_str(value.as_str()) {
                Ok(uri) => Ok(Referer(uri)),
                Err(err) => {
                    log::warn!("failed to parse `Referer` uri: {err}");
                    Err(ErrorStatusCode::BadRequest)
                }
            },
            None => {
                log::warn!("`Referer` header not found");
                Err(ErrorStatusCode::BadRequest)
            }
        }
    }
}
