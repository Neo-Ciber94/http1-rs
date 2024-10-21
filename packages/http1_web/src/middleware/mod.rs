pub mod cors;
pub mod logging;
pub mod sessions;
pub mod timeout;
pub mod redirection;

use http1::{body::Body, request::Request, response::Response};
use std::sync::Arc;

use crate::handler::BoxedHandler;

pub trait Middleware {
    fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body>;
}

impl<F> Middleware for F
where
    F: Fn(Request<Body>, &BoxedHandler) -> Response<Body>,
{
    fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        (self)(req, next)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct BoxedMiddleware(
    Arc<dyn Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static>,
);

impl BoxedMiddleware {
    pub fn new<F>(handler: F) -> Self
    where
        F: Middleware + Send + Sync + 'static,
    {
        BoxedMiddleware(Arc::new(move |req, next| handler.on_request(req, next)))
    }

    pub fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        (self.0)(req, next)
    }
}

impl std::fmt::Debug for BoxedMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedMiddleware").finish()
    }
}