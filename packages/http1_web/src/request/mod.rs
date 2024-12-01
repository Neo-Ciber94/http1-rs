use http1::{body::Body, payload::Payload, request::Request};

use crate::from_request::FromRequest;

/// Request extension methods.
pub trait RequestExt {
    /// Get a value that implements `FromRequestRef`.
    fn extract<U>(&mut self) -> Result<U, U::Rejection>
    where
        U: FromRequest;
}

impl RequestExt for Request<Body> {
    fn extract<U>(&mut self) -> Result<U, U::Rejection>
    where
        U: FromRequest,
    {
        // Split the request in parts
        let body = std::mem::take(self.body_mut());
        let parts = std::mem::take(self.parts_mut());
        let mut payload = Payload::Data(body);

        // Extract the value
        let req = Request::from_parts(parts, ());
        let value = U::from_request(&req, &mut payload)?;

        // Restore request
        let (_, parts) = req.into_parts();
        *self.parts_mut() = parts;
        *self.body_mut() = payload.take().unwrap_or_default();

        Ok(value)
    }
}
