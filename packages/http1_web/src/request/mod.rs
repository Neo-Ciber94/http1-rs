use http1::{body::Body, payload::Payload, request::Request};

use crate::{
    cookies::{Cookie, Cookies},
    from_request::FromRequest,
    middleware::Middleware,
    state::State,
};

/// Request extension methods.
pub trait RequestExt {
    /// Get a request extension from its state.
    fn state<S: Send + Sync + Clone + 'static>(&self) -> Option<S>;

    /// Get a value that implements `FromRequestRef`.
    fn extract<U>(&mut self) -> Result<U, U::Rejection>
    where
        U: FromRequest;
}

impl RequestExt for Request<Body> {
    fn state<S: Send + Sync + Clone + 'static>(&self) -> Option<S> {
        todo!()
    }

    fn extract<U>(&mut self) -> Result<U, U::Rejection>
    where
        U: FromRequest,
    {
        // Butcher the request
        let body = std::mem::take(self.body_mut());
        let parts = std::mem::take(self.parts_mut());
        let mut extensions = std::mem::take(self.extensions_mut());
        let mut payload = Payload::Data(body);
        let req = Request::from_parts(parts, ());

        let value = U::from_request(&req, &mut extensions, &mut payload)?;

        // Restore the request again
        let (_, mut parts) = req.into_parts();
        parts.extensions = extensions;

        *self.parts_mut() = parts;
        *self.body_mut() = payload.take().unwrap_or_default();

        Ok(value)
    }
}
