use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

use http1::{
    body::Body, common::date_time::DateTime, error::BoxError, request::Request, status::StatusCode,
};

use crate::{
    from_request::FromRequestRef,
    into_response::IntoResponse,
    serde::{de::Deserialize, ser::Serialize},
};

#[derive(Clone)]
pub struct Session {
    id: String,
    data: HashMap<String, Box<[u8]>>,
    expires_at: DateTime,
    is_destroyed: Arc<AtomicBool>,
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
        let bytes = match self.data.get(key.as_ref()) {
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
        let bytes = crate::serde::json::to_bytes(&value)?;
        self.data.insert(key.into(), bytes.into_boxed_slice());
        Ok(())
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), BoxError> {
        self.data.remove(key.as_ref());
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
}

pub struct SessionError;
impl IntoResponse for SessionError {
    fn into_response(self) -> http1::response::Response<Body> {
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
