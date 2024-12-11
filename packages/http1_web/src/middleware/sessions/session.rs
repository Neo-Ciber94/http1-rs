use std::{
    collections::HashMap,
    fmt::Display,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use datetime::DateTime;
use http1::{body::Body, error::BoxError, request::Request, status::StatusCode};

use crate::{from_request::FromRequest, IntoResponse};

use serde::{de::Deserialize, impl_serde_enum_str, impl_serde_struct, ser::Serialize};

impl_serde_struct!(Session => {
    id: String,
    inner: Arc<RwLock<SessionInner>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
});

/// Status of this session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session was newly created.
    New,

    /// No status, may be refreshed.
    None,

    /// Session was modified.
    Modified,

    /// Session was destroyed because expired or manually.
    Destroyed,
}

impl_serde_enum_str!(SessionStatus => {
    New,
    None,
    Modified,
    Destroyed
});

struct SessionInner {
    data: HashMap<String, Box<[u8]>>,
    status: SessionStatus,
}

impl_serde_struct!(SessionInner => {
    data: HashMap<String, Box<[u8]>>,
    status: SessionStatus,
});

/// A key-value collection to store information for the current user.
#[derive(Clone)]
pub struct Session {
    id: String,
    inner: Arc<RwLock<SessionInner>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
}

impl Session {
    /// Create a new session.
    ///
    /// This should be done by the session provider and not manually.
    pub fn new(id: impl Into<String>, expires_at: DateTime) -> Self {
        Session {
            id: id.into(),
            expires_at,
            is_destroyed: Arc::new(AtomicBool::new(false)),
            inner: Arc::new(RwLock::new(SessionInner {
                data: Default::default(),
                status: SessionStatus::New,
            })),
        }
    }

    /// Returns the session id.
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    /// Returns the status of this session.
    pub fn status(&self) -> SessionStatus {
        let inner = self.inner.read().expect("failed to lock session data");
        inner.status
    }

    /// Get the value for the given key.
    pub fn get<T: Deserialize>(&self, key: impl AsRef<str>) -> Result<Option<T>, BoxError> {
        let inner = self.inner.read().expect("failed to lock session data");
        let bytes = match inner.data.get(key.as_ref()) {
            Some(x) => x,
            None => return Ok(None),
        };

        let value = serde::json::from_bytes(bytes)?;
        Ok(Some(value))
    }

    /// Returns `true` if the session contain the given value.
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        let inner = self.inner.read().expect("failed to lock session data");
        inner.data.contains_key(key.as_ref())
    }

    /// Reset the session status.
    pub fn refresh_status(&mut self) {
        let mut inner = self.inner.write().expect("failed to lock session data");

        assert!(
            !matches!(inner.status, SessionStatus::Destroyed),
            "cannot refresh the status of a destroyed session"
        );

        inner.status = SessionStatus::None;
    }

    /// Insert or replace the value with the given key.
    pub fn insert<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), BoxError> {
        let mut inner = self.inner.write().expect("failed to lock session data");
        let bytes = serde::json::to_bytes(&value)?;
        inner.data.insert(key.into(), bytes.into_boxed_slice());
        inner.status = SessionStatus::Modified;
        Ok(())
    }

    /// Get the value for the given key or insert it and return it.
    pub fn get_or_insert<T: Serialize + Deserialize>(
        &mut self,
        key: impl AsRef<str>,
        default_value: T,
    ) -> Result<T, BoxError> {
        self.get_or_insert_with(key, || default_value)
    }

    /// Get the value for the given key or insert with the given function it and return it.
    pub fn get_or_insert_with<T: Serialize + Deserialize>(
        &mut self,
        key: impl AsRef<str>,
        f: impl FnOnce() -> T,
    ) -> Result<T, BoxError> {
        if self.contains_key(key.as_ref()) {
            return self.get(key).map(|x| x.expect("value should exists"));
        }

        {
            let mut inner = self.inner.write().expect("failed to lock session data");
            let value = f();
            let bytes = serde::json::to_bytes(&value)?;

            inner
                .data
                .insert(key.as_ref().into(), bytes.into_boxed_slice());

            inner.status = SessionStatus::Modified;
        }

        self.get(key).map(|x| x.expect("value should exists"))
    }

    /// Updates the value associated with the given key or insert it if it don't exists.
    pub fn update_or_insert<T: Serialize + Deserialize, F>(
        &mut self,
        key: impl AsRef<str>,
        initial: T,
        updater: F,
    ) -> Result<(), BoxError>
    where
        F: FnOnce(T) -> T,
    {
        self.update_or_insert_with(key, || initial, updater)
    }

    /// Updates the value associated with the given key or insert with a function it if it don't exists.
    pub fn update_or_insert_with<T, U, F>(
        &mut self,
        key: impl AsRef<str>,
        initial: U,
        updater: F,
    ) -> Result<(), BoxError>
    where
        F: FnOnce(T) -> T,
        U: FnOnce() -> T,
        T: Serialize + Deserialize,
    {
        if !self.contains_key(key.as_ref()) {
            return self.insert(key.as_ref().to_owned(), initial());
        }

        let value = match self.get::<T>(key.as_ref())? {
            Some(x) => x,
            None => initial(),
        };

        let new_value = updater(value);
        self.insert(key.as_ref().to_owned(), new_value)?;

        Ok(())
    }

    /// Removes the value associated with the given key.
    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), BoxError> {
        let mut inner = self.inner.write().expect("failed to lock session data");
        inner.data.remove(key.as_ref());
        inner.status = SessionStatus::Modified;
        Ok(())
    }

    /// Whether if this session is expired.
    pub fn is_expired(&self) -> bool {
        DateTime::now_utc() > self.expires_at
    }

    /// Remove this session from this user.
    pub fn destroy(self) {
        self.is_destroyed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Whether if this session is destroyed for this user.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Whether if this session is expired or destroyed.
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_destroyed()
    }
}

#[derive(Debug)]
pub struct SessionError;

impl Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to retrieve session")
    }
}

impl std::error::Error for SessionError {}

impl IntoResponse for SessionError {
    fn into_response(self) -> http1::response::Response<Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequest for Session {
    type Rejection = SessionError;

    fn from_request(
        req: &Request<()>,
        _payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<Session>()
            .cloned()
            .ok_or(SessionError)
    }
}
