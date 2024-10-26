use std::{
    borrow::Borrow,
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use http1::headers::Headers;

use crate::{from_request::FromRequest, into_response::IntoResponse};

mod accept;
mod authorization;
mod entity;
mod host;
mod referer;
mod user_agent;

pub use {accept::*, authorization::*, entity::*, host::*, referer::*, user_agent::*};

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

impl<H: FromHeaders> Deref for GetHeader<H> {
    type Target = H;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<H: FromHeaders> DerefMut for GetHeader<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<H: FromHeaders> Borrow<H> for GetHeader<H> {
    fn borrow(&self) -> &H {
        &self.0
    }
}

impl<H: FromHeaders> GetHeader<H> {
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<H: FromHeaders> FromRequest for GetHeader<H> {
    type Rejection = H::Rejection;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let header = H::from_headers(req.headers())?;
        Ok(GetHeader(header))
    }
}
