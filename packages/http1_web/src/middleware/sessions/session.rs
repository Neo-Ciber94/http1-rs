use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use http1::{
    body::Body, common::date_time::DateTime, error::BoxError, request::Request, status::StatusCode,
};

use crate::{
    from_request::FromRequestRef,
    impl_serde_struct,
    into_response::IntoResponse,
    serde::{de::Deserialize, ser::Serialize},
};

impl_serde_struct!(Session => {
    id: String,
    data: Arc<RwLock<HashMap<String, Box<[u8]>>>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
});

#[derive(Clone)]
pub struct Session {
    id: String,
    data: Arc<RwLock<HashMap<String, Box<[u8]>>>>,
    is_destroyed: Arc<AtomicBool>,
    expires_at: DateTime,
}

impl Session {
    pub fn new(id: impl Into<String>, expires_at: DateTime) -> Self {
        Session {
            id: id.into(),
            data: Default::default(),
            expires_at,
            is_destroyed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn get<T: Deserialize>(&self, key: impl AsRef<str>) -> Result<Option<T>, BoxError> {
        let data = self.data.read().expect("failed to lock session data");
        let bytes = match data.get(key.as_ref()) {
            Some(x) => x,
            None => return Ok(None),
        };

        let value = crate::serde::json::from_bytes(bytes)?;
        Ok(Some(value))
    }

    pub fn insert<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), BoxError> {
        let mut data = self.data.write().expect("failed to lock session data");
        let bytes = crate::serde::json::to_bytes(&value)?;
        data.insert(key.into(), bytes.into_boxed_slice());
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
        let mut data = self.data.write().expect("failed to lock session data");
        if data.contains_key(key.as_ref()) {
            drop(data);
            return self.get(key).map(|x| x.expect("value should exists"));
        }

        let value = f();
        let bytes = crate::serde::json::to_bytes(&value)?;
        data.insert(key.as_ref().into(), bytes.into_boxed_slice());

        drop(data);
        self.get(key).map(|x| x.expect("value should exists"))
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), BoxError> {
        let mut data = self.data.write().expect("failed to lock session data");
        data.remove(key.as_ref());
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

impl IntoResponse for SessionError {
    fn into_response(self) -> http1::response::Response<Body> {
        eprintln!("Failed to retrieve session");
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
