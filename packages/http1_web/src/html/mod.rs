use http1::{headers::CONTENT_TYPE, response::Response};

use crate::IntoResponse;

mod dsl;
pub mod element;

pub use dsl::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Html<T>(pub T);

impl<T> Html<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> IntoResponse for Html<T>
where
    T: Into<String>,
{
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let body = self.0.into();
        Response::builder()
            .insert_header(CONTENT_TYPE, "text/html")
            .body(body.into())
    }
}
