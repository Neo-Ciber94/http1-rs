use std::sync::Arc;

use crate::{
    body::Body,
    http::{request::Request, response::Response},
};

/// Handles a server request.
pub trait RequestHandler {
    /// Gets a request and returns a response.
    fn handle(&self, req: Request<Body>) -> Response<Body>;
}

impl<F: Fn(Request<Body>) -> Response<Body>> RequestHandler for F {
    fn handle(&self, req: Request<Body>) -> Response<Body> {
        (self)(req)
    }
}

impl<T: RequestHandler> RequestHandler for Arc<T> {
    fn handle(&self, req: Request<Body>) -> Response<Body> {
        self.as_ref().handle(req)
    }
}
