use http1::request::Request;

use crate::{
    cookies::{Cookie, Cookies},
    from_request::FromRequestRef,
    middleware::Middleware,
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
}

impl<T> RequestExt for Request<T> {
    fn get_all_cookies(&self) -> &Cookies {
        let ext = self
            .extensions()
            .get::<Ext<Cookies>>()
            .expect("`RequestExtensions` middleware was not set or extensions where removed");

        &ext.0
    }
}
