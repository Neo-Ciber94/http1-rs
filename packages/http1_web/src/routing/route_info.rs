use std::{convert::Infallible, fmt::Display, ops::Deref};

use crate::from_request::FromRequestRef;

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

impl FromRequestRef for RouteInfo {
    type Rejection = Infallible;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        Ok(req
            .extensions()
            .get::<RouteInfo>()
            .cloned()
            .expect("failed to get request RouteInfo"))
    }
}