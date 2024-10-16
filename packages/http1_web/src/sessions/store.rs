use std::{collections::HashMap, sync::Arc, time::Duration};

use http1::{common::date_time::DateTime, error::BoxError};

use super::session::Session;

/// Provides a way to create, update, read and delete sessions.
pub trait SessionStore {
    /// Get or create a session for the given session id.
    fn load_session(&mut self, session_id: &str) -> Result<Session, BoxError>;

    /// Saves a session.
    fn save_session(&mut self, session: Session) -> Result<(), BoxError>;

    /// Destroy a session.
    fn destroy_session(&mut self, session: Session) -> Result<(), BoxError>;
}

pub struct MemoryStore(HashMap<Arc<str>, Session>);

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore(Default::default())
    }
}

impl SessionStore for MemoryStore {
    fn load_session(&mut self, session_id: &str) -> Result<Session, BoxError> {
        match self.0.get(session_id) {
            Some(session) => Ok(session.clone()),
            None => {
                let session_id: Arc<str> = session_id.into();
                let expires_at = DateTime::now_utc() + Duration::from_secs(60);
                let session = Session::new(session_id.as_ref(), expires_at);
                self.0.insert(session_id.clone(), session.clone());
                Ok(session)
            }
        }
    }

    fn save_session(&mut self, session: Session) -> Result<(), BoxError> {
        match self.0.get_mut(session.id()) {
            Some(x) => {
                *x = session;
            }
            None => {
                self.0.insert(session.id().into(), session);
            }
        }

        Ok(())
    }

    fn destroy_session(&mut self, session: Session) -> Result<(), BoxError> {
        if !session.is_destroyed() {
            return Err(String::from("Session was not marked as destroyed").into());
        }

        self.0.remove(session.id());

        Ok(())
    }
}
