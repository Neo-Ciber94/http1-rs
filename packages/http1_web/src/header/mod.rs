use std::convert::Infallible;

use http1::headers::Headers;

use crate::into_response::IntoResponse;

mod accept;
mod entity;
mod host;
mod referer;
mod user_agent;

pub use {accept::*, entity::*, host::*, referer::*, user_agent::*};

/// Allow to create a value from the request headers.
pub trait FromHeaders: Sized {
    type Rejection: IntoResponse;

    /// Create an instance of the headers.
    fn from_headers(headers: &Headers) -> Result<Self, Self::Rejection>;
}

impl<T: FromHeaders> FromHeaders for Option<T> {
    type Rejection = Infallible;

    fn from_headers(headers: &Headers) -> Result<Self, Self::Rejection> {
        match T::from_headers(headers) {
            Ok(x) => Ok(Some(x)),
            Err(_) => Ok(None),
        }
    }
}

impl FromHeaders for Headers {
    type Rejection = Infallible;

    fn from_headers(headers: &Headers) -> Result<Self, Self::Rejection> {
        Ok(headers.clone())
    }
}

/// Extracts a typed request header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetHeader<H: FromHeaders>(pub H);

impl<H: FromHeaders> GetHeader<H> {
    pub fn into_inner(self) -> H {
        self.0
    }
}
