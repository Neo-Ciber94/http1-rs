use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use http1::{body::Body, headers, request::Request, rng::random::Alphanumeric, status::StatusCode};

use crate::{
    cookies::{Cookie, Cookies},
    from_request::FromRequestRef,
    into_response::IntoResponse,
    middleware::Middleware,
};

use super::store::{LoadSessionConfig, SessionStore};

pub type IdGenerator = fn(&Request<Body>) -> String;

pub struct SessionProvider<S> {
    store: Arc<Mutex<S>>,
    cookie_name: String,
    id_generator: IdGenerator,
    max_age: u64,
    path: String,
    is_http_only: bool,
}

impl SessionProvider<()> {
    pub fn new() -> Builder {
        Builder::new(generate_session_id)
    }
}

pub struct Builder {
    cookie_name: String,
    id_generator: IdGenerator,
    max_age: u64,
    is_http_only: bool,
    path: String,
}

impl Builder {
    pub fn new(id_generator: IdGenerator) -> Self {
        Builder {
            id_generator,
            cookie_name: "session_id".into(),
            path: "/".into(),
            is_http_only: true,
            max_age: 60,
        }
    }

    pub fn cookie_name(mut self, cookie_name: impl Into<String>) -> Self {
        self.cookie_name = cookie_name.into();
        self
    }

    pub fn id_generator(mut self, id_generator: IdGenerator) -> Self {
        self.id_generator = id_generator;
        self
    }

    pub fn max_age(mut self, max_age: u64) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn store<S>(self, store: S) -> SessionProvider<S> {
        SessionProvider {
            id_generator: self.id_generator,
            cookie_name: self.cookie_name,
            max_age: self.max_age,
            path: self.path,
            is_http_only: self.is_http_only,
            store: Arc::new(Mutex::new(store)),
        }
    }
}

impl<S: SessionStore> Middleware for SessionProvider<S> {
    fn on_request(
        &self,
        mut req: Request<Body>,
        next: &crate::handler::BoxedHandler,
    ) -> http1::response::Response<Body> {
        let mut store = match self.store.lock() {
            Ok(x) => x,
            Err(_) => {
                log::error!("Failed to lock session store");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let cookies = match Cookies::from_request_ref(&req) {
            Ok(x) => x,
            Err(_) => {
                log::error!("Failed to parse cookies");
                Cookies::default()
            }
        };

        let mut is_new_session = false;
        let session_id = match cookies.get(&self.cookie_name) {
            Some(x) => x.value().to_owned(),
            None => {
                is_new_session = true;
                (self.id_generator)(&req)
            }
        };

        let config = LoadSessionConfig {
            duration: Duration::from_secs(self.max_age),
        };

        let session = match store.load_session(&session_id, &config) {
            Ok(x) => x,
            Err(_) => {
                log::error!("Failed to load session");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        // Add the session to extensions
        req.extensions_mut().insert(session.clone());
        drop(store);

        // Return the response
        let mut response = next.call(req);

        match self.store.lock() {
            Ok(mut store) => {
                // Check if the session is expired or destroyed, if so destroy it
                if session.is_destroyed() || session.is_expired() {
                    if let Err(err) = store.destroy_session(session) {
                        log::error!("Failed to destroy session: {err}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                } else {
                    if let Err(err) = store.save_session(session) {
                        log::error!("Failed to save session: {err}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
            Err(_) => {
                log::error!("Failed to lock session store");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        // Sets the session cookie
        if is_new_session {
            let cookie = Cookie::new(self.cookie_name.as_str(), session_id)
                .path(self.path.as_str())
                .http_only(self.is_http_only)
                .max_age(self.max_age)
                .build();

            response
                .headers_mut()
                .append(headers::SET_COOKIE, cookie.to_string());
        }

        response
    }
}

fn generate_session_id(_req: &Request<Body>) -> String {
    http1::rng::sequence::<Alphanumeric>()
        .take(36)
        .collect::<String>()
}
