use std::{collections::HashMap, sync::Arc, time::Duration};

use super::session::Session;
use datetime::DateTime;
use http1::error::BoxError;

/// Configuration used for loading sessions.
pub struct LoadSessionConfig {
    /// The duration for a new created session.
    pub duration: Duration,
}

/// Provides a way to create, update, read and delete sessions.
pub trait SessionStore {
    /// If a session exists and is valid return it, otherwise create a new session using the given config.
    fn load_session(
        &mut self,
        session_id: &str,
        config: &LoadSessionConfig,
    ) -> Result<Session, BoxError>;

    /// Adds a session to the store.
    fn save_session(&mut self, session: Session) -> Result<(), BoxError>;

    /// Removes a session from the store.
    fn destroy_session(&mut self, session: Session) -> Result<(), BoxError>;
}

/// A store that save sessions in memory.
pub struct MemoryStore(HashMap<Arc<str>, Session>);

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore(Default::default())
    }
}

impl MemoryStore {
    fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.0.get(session_id)
    }

    fn create_session(&mut self, session_id: &str, config: &LoadSessionConfig) -> Session {
        let session_id: Arc<str> = session_id.into();
        let expires_at = DateTime::now_utc() + config.duration;
        let session = Session::new(session_id.as_ref(), expires_at);
        self.0.insert(session_id.clone(), session.clone());
        session
    }
}

impl SessionStore for MemoryStore {
    fn load_session(
        &mut self,
        session_id: &str,
        config: &LoadSessionConfig,
    ) -> Result<Session, BoxError> {
        match self.get_session(session_id).filter(|x| x.is_valid()) {
            Some(session) => Ok(session.clone()),
            None => Ok(self.create_session(session_id, config)),
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
