use std::{ops::Deref, sync::Arc};

use crate::from_request::FromRequestRef;

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

impl<T: 'static> FromRequestRef for State<T> {
    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, http1::error::BoxError> {
        req.extensions()
            .get::<State<T>>()
            .cloned()
            .ok_or_else(|| format!("failed to get app State<T>").into())
    }
}
