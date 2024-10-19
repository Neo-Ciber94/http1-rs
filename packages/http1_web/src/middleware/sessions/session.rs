use std::{
    collections::HashMap,
    fmt::Display,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use datetime::DateTime;
use http1::{body::Body, error::BoxError, request::Request, status::StatusCode};

use crate::{
    from_request::FromRequestRef,
    impl_serde_enum_str, impl_serde_struct,
    into_response::IntoResponse,
    serde::{de::Deserialize, ser::Serialize},
};

impl_serde_struct!(Session => {
    id: String,
    inner: Arc<RwLock<SessionInner>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    New,
    None,
    Modified,
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

#[derive(Clone)]
pub struct Session {
    id: String,
    inner: Arc<RwLock<SessionInner>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
}

impl Session {
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

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn status(&self) -> SessionStatus {
        let inner = self.inner.read().expect("failed to lock session data");
        inner.status
    }

    pub fn get<T: Deserialize>(&self, key: impl AsRef<str>) -> Result<Option<T>, BoxError> {
        let inner = self.inner.read().expect("failed to lock session data");
        let bytes = match inner.data.get(key.as_ref()) {
            Some(x) => x,
            None => return Ok(None),
        };

        let value = crate::serde::json::from_bytes(bytes)?;
        Ok(Some(value))
    }

    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        let inner = self.inner.read().expect("failed to lock session data");
        inner.data.contains_key(key.as_ref())
    }

    pub fn refresh_status(&mut self) {
        let mut inner = self.inner.write().expect("failed to lock session data");

        assert!(
            !matches!(inner.status, SessionStatus::Destroyed),
            "cannot refresh the status of a destroyed session"
        );

        inner.status = SessionStatus::None;
    }

    pub fn insert<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), BoxError> {
        let mut inner = self.inner.write().expect("failed to lock session data");
        let bytes = crate::serde::json::to_bytes(&value)?;
        inner.data.insert(key.into(), bytes.into_boxed_slice());
        inner.status = SessionStatus::Modified;
        Ok(())
    }

    pub fn get_or_insert<T: Serialize + Deserialize>(
        &mut self,
        key: impl AsRef<str>,
        default_value: T,
    ) -> Result<T, BoxError> {
        self.get_or_insert_with(key, || default_value)
    }

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
            let bytes = crate::serde::json::to_bytes(&value)?;

            inner
                .data
                .insert(key.as_ref().into(), bytes.into_boxed_slice());

            inner.status = SessionStatus::Modified;
        }

        self.get(key).map(|x| x.expect("value should exists"))
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), BoxError> {
        let mut inner = self.inner.write().expect("failed to lock session data");
        inner.data.remove(key.as_ref());
        inner.status = SessionStatus::Modified;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        DateTime::now_utc() > self.expires_at
    }

    pub fn destroy(self) {
        self.is_destroyed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(std::sync::atomic::Ordering::Acquire)
    }

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

impl FromRequestRef for Session {
    type Rejection = SessionError;

    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<Session>()
            .cloned()
            .ok_or_else(|| SessionError)
    }
}
