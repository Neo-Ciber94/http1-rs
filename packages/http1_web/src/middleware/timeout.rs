use std::{sync::mpsc::RecvTimeoutError, time::Duration};

use http1::{body::Body, request::Request, response::Response, status::StatusCode};

use crate::{handler::BoxedHandler, IntoResponse};

use super::Middleware;

pub struct Timeout {
    timeout: Duration,
}

impl Timeout {
    pub fn new(timeout: Duration) -> Self {
        Timeout { timeout }
    }
}

impl Middleware for Timeout {
    fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let timeout = self.timeout;
        let next = next.clone();

        std::thread::spawn(move || {
            let response = next.call(req);
            let _ = sender.send(response);
        });

        match receiver.recv_timeout(timeout) {
            Ok(response) => response,
            Err(RecvTimeoutError::Disconnected) => {
                log::error!("Timeout sender disconnected");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Err(RecvTimeoutError::Timeout) => Response::builder()
                .status(StatusCode::REQUEST_TIMEOUT)
                .body(Body::empty()),
        }
    }
}
