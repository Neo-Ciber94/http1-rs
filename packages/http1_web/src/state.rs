use std::{fmt::Display, ops::Deref, sync::Arc};

use http1::status::StatusCode;

use crate::{from_request::FromRequestRef, into_response::IntoResponse};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State<T>(Arc<T>);

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        State(Arc::new(value))
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct AppStateError;

impl Display for AppStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to extract app state")
    }
}

impl IntoResponse for AppStateError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<T: Send + Sync + 'static> FromRequestRef for State<T> {
    type Rejection = AppStateError;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<State<T>>()
            .cloned()
            .ok_or(AppStateError)
    }
}
