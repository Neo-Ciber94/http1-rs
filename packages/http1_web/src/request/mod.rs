use http1::{body::Body, request::Request};

use crate::{
    cookies::{Cookie, Cookies},
    from_request::FromRequestRef,
    middleware::Middleware,
    state::State,
};

struct Ext<T>(T);

/// Add extra extensions for the [`Request<T>`].
#[derive(Debug)]
pub struct RequestExtensions;

impl Middleware for RequestExtensions {
    fn on_request(
        &self,
        mut req: Request<http1::body::Body>,
        next: &crate::handler::BoxedHandler,
    ) -> http1::response::Response<http1::body::Body> {
        let cookies = Cookies::from_request_ref(&req).expect("failed to parse cookies");

        // Insert any cookie extensions
        req.extensions_mut().insert(Ext(cookies));

        // Return the response
        next.call(req)
    }
}

/// Request extension methods, this requires to add the [`RequestExtensions`] middleware.
pub trait RequestExt {
    /// Returns all the request cookies.
    fn get_all_cookies(&self) -> &Cookies;

    /// Return a cookie with the given name.
    fn cookie(&self, name: impl AsRef<str>) -> Option<&Cookie> {
        self.get_all_cookies().get(name)
    }

    /// Get a request extension from its state.
    fn state<T: Send + Sync + Clone + 'static>(&self) -> Option<T>;

    /// Get a value that implements `FromRequestRef`.
    fn extract<T>(&self) -> Result<T, T::Rejection>
    where
        T: FromRequestRef;
}

impl RequestExt for Request<Body> {
    fn get_all_cookies(&self) -> &Cookies {
        let ext = self
            .extensions()
            .get::<Ext<Cookies>>()
            .expect("`RequestExtensions` middleware was not set or extensions where removed");

        &ext.0
    }

    fn state<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
        State::<T>::from_request_ref(self).ok().map(|x| x.0.clone())
    }

    fn extract<T>(&self) -> Result<T, T::Rejection>
    where
        T: FromRequestRef,
    {
        T::from_request_ref(self)
    }
}
