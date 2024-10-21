use crate::{into_response::IntoResponse, redirect::Redirect};

use super::Middleware;

pub struct Redirection {
    from: String,
    to: String,
}

impl Redirection {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        let from = from.into();
        let to = to.into();

        assert!(from.starts_with("/"), "`from` route should start with `/`");

        if from.len() > 1 {
            assert!(!from.ends_with("/"), "`from` route should not end with `/`");
        }

        assert!(to.starts_with("/"), "`from` route should start with `/`");

        if to.len() > 1 {
            assert!(!to.ends_with("/"), "`from` route should not end with `/`");
        }


        Redirection { from, to }
    }
}

impl Middleware for Redirection {
    fn on_request(
        &self,
        req: http1::request::Request<http1::body::Body>,
        next: &crate::handler::BoxedHandler,
    ) -> http1::response::Response<http1::body::Body> {
        let path = req.uri().path_and_query().path();

        if path == self.from {
            return Redirect::see_other(&self.to).into_response();
        }

        next.call(req)
    }
}
