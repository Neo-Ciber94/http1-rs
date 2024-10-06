pub mod sse;

use super::{
    common::any_map::AnyMap,
    headers::{HeaderName, HeaderValue, Headers},
    status::StatusCode,
    version::Version,
};

/// Represents an HTTP response.
///
/// This struct holds the HTTP status code, headers, and body of the response.
/// The body type is generic, allowing different types of body content.
#[derive(Debug)]
pub struct Response<T> {
    status: StatusCode,
    headers: Headers,
    extensions: AnyMap,
    body: T,
}

impl<T> Response<T> {
    /// Creates a new `Response` with the given status code and body.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code for the response.
    /// - `body`: The content of the response body.
    ///
    /// # Returns
    /// A `Response` with the provided status and body, and empty headers.
    pub fn new(status: StatusCode, body: T) -> Self {
        Response {
            status,
            body,
            headers: Headers::new(),
            extensions: Default::default(),
        }
    }

    /// Returns the status code of the response.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns a mutable reference to the status code of the response.
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    /// Returns a reference to the response headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a mutable reference to the response headers.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Returns the HTTP version of the response.
    ///
    /// Currently, this always returns `Version::Http1_1`.
    pub fn version(&self) -> Version {
        Version::Http1_1
    }

    /// Returns a reference to the response body.
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the response body.
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    /// Returns a reference to the response body.
    pub fn extensions(&self) -> &AnyMap {
        &self.extensions
    }

    /// Returns a mutable reference to the response extensions.
    pub fn extensions_mut(&mut self) -> &mut AnyMap {
        &mut self.extensions
    }

    /// Maps this response body
    pub fn map_body<F: FnOnce(T) -> R, R>(self, f: F) -> Response<R> {
        let new_body = f(self.body);
        Response {
            body: new_body,
            headers: self.headers,
            status: self.status,
            extensions: self.extensions,
        }
    }

    /// Consumes the response and splits it into its components:
    /// the status code, headers, and body.
    ///
    /// # Returns
    /// A tuple containing the status code, headers, and body.
    pub fn into_parts(self) -> (StatusCode, Headers, T) {
        let Self {
            status,
            headers,
            body,
            extensions: _,
        } = self;

        (status, headers, body)
    }
}

impl Response<()> {
    /// Returns a `Builder` to construct a new `Response`.
    ///
    /// # Example
    /// ```
    /// use http1::{status::StatusCode, response::Response};
    ///
    /// let response = Response::builder()
    ///     .status(StatusCode::OK)
    ///     .build("Hello, World!");
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// A builder for constructing `Response` objects.
///
/// The `Builder` allows setting the status code and headers before
/// building the final response with a body.
#[derive(Default)]
pub struct Builder {
    status: StatusCode,
    headers: Headers,
    extensions: AnyMap,
}

impl Builder {
    /// Creates a new `Builder` with a default status code of `200 OK` and empty headers.
    pub fn new() -> Self {
        Builder {
            status: StatusCode::OK,
            headers: Headers::new(),
            extensions: Default::default(),
        }
    }

    /// Sets the status code for the response.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code to set.
    ///
    /// # Returns
    /// The updated `Builder` with the new status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Returns a reference to the headers being set in the `Builder`.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a mutable reference to the headers being set in the `Builder`.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Returns a reference to the extensions being set in the `Builder`.
    pub fn extensions(&self) -> &AnyMap {
        &self.extensions
    }

    /// Returns a mutable reference to the extensions being set in the `Builder`.
    pub fn extensions_mut(&mut self) -> &mut AnyMap {
        &mut self.extensions
    }

    /// Inserts a new header into the response.
    ///
    /// If the header already exists, it is replaced.
    ///
    /// # Parameters
    /// - `key`: The name of the header.
    /// - `value`: The value of the header.
    ///
    /// # Returns
    /// The updated `Builder` with the new header.
    pub fn insert_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.insert(key.into(), value);
        self
    }

    /// Appends a value to an existing header in the response.
    ///
    /// If the header does not exist, it is created.
    ///
    /// # Parameters
    /// - `key`: The name of the header.
    /// - `value`: The value to append to the header.
    ///
    /// # Returns
    /// The updated `Builder` with the appended header.
    pub fn append_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.append(key.into(), value);
        self
    }

    /// Builds the final `Response` with the provided body.
    ///
    /// # Parameters
    /// - `body`: The content to include in the response body.
    ///
    /// # Returns
    /// A `Response` containing the status, headers, and body.
    pub fn build<T>(self, body: T) -> Response<T> {
        let Self {
            status,
            headers,
            extensions,
        } = self;

        Response {
            status,
            headers,
            extensions,
            body,
        }
    }
}
