use http1::extensions::Extensions;

use super::Middleware;

/// Allow to insert values into the request extensions.
#[derive(Default)]
pub struct ExtensionsProvider(Extensions);

impl ExtensionsProvider {
    /// Constructs a new request extension provider.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a value to the extensions.
    pub fn insert<T>(mut self, value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0.insert(value);
        self
    }
}

impl Middleware for ExtensionsProvider {
    fn on_request(
        &self,
        mut req: http1::request::Request<http1::body::Body>,
        next: &crate::handler::BoxedHandler,
    ) -> http1::response::Response<http1::body::Body> {
        // Add the extensions
        req.extensions_mut().extend(self.0.clone());

        // Call next
        next.call(req)
    }
}
