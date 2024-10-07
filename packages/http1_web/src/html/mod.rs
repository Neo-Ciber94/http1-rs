use http1::{headers::CONTENT_LENGTH, response::Response};

use crate::into_response::IntoResponse;

pub mod element;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Html(String);

impl Html {
    pub fn raw(s: impl Into<String>) -> Self {
        Html(s.into())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl IntoResponse for Html {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        Response::builder()
            .insert_header(CONTENT_LENGTH, "text/html")
            .body(self.0.into())
    }
}
