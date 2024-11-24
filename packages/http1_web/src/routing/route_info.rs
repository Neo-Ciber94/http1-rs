use std::{convert::Infallible, fmt::Display, ops::Deref};

use crate::from_request::FromRequest;

use super::route::Route;

/// Provides route information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteInfo(pub(crate) Route);

impl RouteInfo {
    pub fn into_inner(self) -> Route {
        self.0
    }
}

impl Display for RouteInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for RouteInfo {
    type Target = Route;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for RouteInfo {
    type Rejection = Infallible;

    fn from_request(
        req: &http1::request::Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(extensions
            .get::<RouteInfo>()
            .cloned()
            .expect("failed to get request RouteInfo"))
    }
}
