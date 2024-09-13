use super::{
    headers::{HeaderName, Headers},
    status::StatusCode,
    version::Version,
};

#[derive(Debug)]
pub struct Response<T> {
    status: StatusCode,
    headers: Headers,
    body: T,
}

impl<T> Response<T> {
    pub fn new(status: StatusCode, body: T) -> Self {
        Response {
            status,
            body,
            headers: Headers::new(),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    pub fn version(&self) -> Version {
        Version::Http1_1
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    pub fn into_parts(self) -> (StatusCode, Headers, T) {
        let Self {
            status,
            headers,
            body,
        } = self;

        (status, headers, body)
    }
}

impl Response<()> {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}

pub struct ResponseBuilder {
    status: StatusCode,
    headers: Headers,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            status: StatusCode::OK,
            headers: Headers::new(),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    pub fn insert_header<K: Into<HeaderName>>(mut self, key: K, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value);
        self
    }

    pub fn append_header<K: Into<HeaderName>>(mut self, key: K, value: impl Into<String>) -> Self {
        self.headers.append(key.into(), value);
        self
    }

    pub fn build<T>(self, body: T) -> Response<T> {
        let Self { status, headers } = self;

        Response {
            status,
            headers,
            body,
        }
    }
}
