use http1::{body::Body, request::Request, response::Response};

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
pub struct BoxedMiddleware(
    Box<dyn Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static>,
);

impl BoxedMiddleware {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static,
    {
        BoxedMiddleware(Box::new(move |req, next| handler.on_request(req, next)))
    }

    pub fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        (self.0)(req, next)
    }
}
