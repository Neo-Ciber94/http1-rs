pub mod cors;
pub mod logging;
pub mod redirection;
pub mod sessions;
pub mod timeout;

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
pub struct BoxedMiddleware {
    inner: Arc<dyn Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static>,

    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl BoxedMiddleware {
    pub fn new<F>(handler: F) -> Self
    where
        F: Middleware + Send + Sync + 'static,
    {
        BoxedMiddleware {
            inner: Arc::new(move |req, next| handler.on_request(req, next)),

            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<F>(),
        }
    }

    pub fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        (self.inner)(req, next)
    }
}

impl std::fmt::Debug for BoxedMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            f.debug_tuple("BoxedMiddleware")
                .field(&self.type_name)
                .finish()
        } else {
            f.debug_struct("BoxedMiddleware").finish_non_exhaustive()
        }
    }
}
