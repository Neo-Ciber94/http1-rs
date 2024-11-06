use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use http1::{common::any_map::AnyMap, status::StatusCode};

use crate::{from_request::FromRequestRef, IntoResponse};

/// A state to share within requests.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State<T>(pub T);

impl<T> State<T> {
    /// Constructs a new state.
    pub fn new(value: T) -> Self {
        State(value)
    }

    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct AppState(AnyMap);

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AppState").finish()
    }
}

impl Deref for AppState {
    type Target = AnyMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AppState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct AppStateError<T>(PhantomData<T>);

impl<T> Display for AppStateError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to extract app state for: {}",
            std::any::type_name::<T>()
        )
    }
}

impl<T> IntoResponse for AppStateError<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<T: Clone + Send + Sync + 'static> FromRequestRef for State<T> {
    type Rejection = AppStateError<T>;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<Arc<AppState>>()
            .and_then(|x| x.get::<T>())
            .cloned()
            .map(State)
            .ok_or(AppStateError(PhantomData))
    }
}
