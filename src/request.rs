use std::fmt::Display;

use super::{
    headers::{HeaderName, HeaderValue, Headers},
    method::Method,
    uri::{
        path_query::PathAndQuery,
        uri::{InvalidUri, Uri},
    },
    version::Version,
};

/// Represents an HTTP request.
///
/// This struct holds the HTTP method, version, URL, headers, and body of the request.
/// The body is generic, allowing flexibility in request content.
#[derive(Debug)]
pub struct Request<T> {
    headers: Headers,
    method: Method,
    version: Version,
    uri: Uri,
    body: T,
}

impl<T> Request<T> {
    /// Creates a new `Request` with the specified method, version, URL, and body.
    ///
    /// # Parameters
    /// - `method`: The HTTP method of the request (e.g., `GET`, `POST`).
    /// - `version`: The HTTP version used (e.g., `HTTP/1.1`).
    /// - `uri`: The URL of the request.
    /// - `body`: The content of the request body.
    ///
    /// # Returns
    /// A new `Request` instance with empty headers.
    pub fn new(method: Method, version: Version, uri: Uri, body: T) -> Self {
        Request {
            body,
            method,
            version,
            uri,
            headers: Headers::default(),
        }
    }

    /// Returns a reference to the HTTP method of the request.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns a mutable reference to the HTTP method of the request.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    /// Returns a reference to the HTTP version of the request.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns a mutable reference to the HTTP version of the request.
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    /// Returns a reference to the URL of the request.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Returns a reference to the headers of the request.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a mutable reference to the headers of the request.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Returns a reference to the body of the request.
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the body of the request.
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }
}

impl Request<()> {
    /// Returns a `Builder` for constructing a new `Request`.
    ///
    /// # Example
    /// ```
    /// use http1::http::{method::Method,request::Request};
    ///
    /// let request = Request::builder()
    ///     .method(Method::POST)
    ///     .uri("/submit")
    ///     .build("body content")
    ///     .unwrap();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
}

struct Parts {
    headers: Headers,
    method: Method,
    version: Version,
    uri: Uri,
}

#[derive(Debug)]
pub enum InvalidRequest {
    InvalidUri(InvalidUri),
}

impl Display for InvalidRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidRequest::InvalidUri(invalid_uri) => write!(f, "{invalid_uri}"),
        }
    }
}

impl std::error::Error for InvalidRequest {}

/// A builder for constructing `Request` objects.
///
/// The `Builder` allows setting the HTTP method, version, URL, and headers before
/// building the final `Request` with a body.
pub struct Builder {
    inner: Result<Parts, InvalidRequest>,
}

impl Builder {
    /// Creates a new `Builder` with default values:
    /// - `GET` method
    /// - `HTTP/1.1` version
    /// - URL set to "/"
    /// - Empty headers
    pub fn new() -> Self {
        let parts = Parts {
            headers: Headers::new(),
            method: Method::GET,
            uri: Uri::new(None, None, PathAndQuery::new("/".to_owned(), None, None)),
            version: Version::Http1_1,
        };

        Builder { inner: Ok(parts) }
    }

    fn update<F: FnOnce(&mut Parts) -> Result<(), InvalidRequest>>(mut self, f: F) -> Self {
        self.inner = self.inner.and_then(move |mut parts| {
            f(&mut parts)?;
            Ok(parts)
        });

        self
    }

    /// Sets the HTTP version of the request.
    ///
    /// # Parameters
    /// - `version`: The HTTP version to set (e.g., `Version::Http1_1`).
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn version(self, version: Version) -> Self {
        self.update(|parts| {
            parts.version = version;
            Ok(())
        })
    }

    /// Returns a mutable reference to the HTTP version.
    pub fn version_mut(&mut self) -> Option<&mut Version> {
        self.inner.as_mut().ok().map(|parts| &mut parts.version)
    }

    /// Sets the HTTP method of the request.
    ///
    /// # Parameters
    /// - `method`: The HTTP method to set (e.g., `Method::POST`).
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn method(self, method: Method) -> Self {
        self.update(|parts| {
            parts.method = method;
            Ok(())
        })
    }

    /// Returns a mutable reference to the HTTP method.
    pub fn method_mut(&mut self) -> Option<&mut Method> {
        self.inner.as_mut().ok().map(|parts| &mut parts.method)
    }

    /// Sets the URL of the request.
    ///
    /// # Parameters
    /// - `url`: The URL to set.
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn uri<T>(self, uri: T) -> Self
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.update(|parts| {
            parts.uri = uri
                .try_into()
                .map_err(|err| InvalidRequest::InvalidUri(err.into()))?;
            Ok(())
        })
    }

    /// Returns a mutable reference to the URL.
    pub fn uri_mut(&mut self) -> Option<&mut Uri> {
        self.inner.as_mut().ok().map(|parts| &mut parts.uri)
    }

    /// Returns a reference to the headers being set in the `Builder`.
    pub fn headers(&self) -> Option<&Headers> {
        self.inner.as_ref().ok().map(|parts| &parts.headers)
    }

    /// Returns a mutable reference to the headers being set in the `Builder`.
    pub fn headers_mut(&mut self) -> Option<&mut Headers> {
        self.inner.as_mut().ok().map(|parts| &mut parts.headers)
    }

    /// Inserts a header into the request.
    ///
    /// If the header already exists, it is replaced.
    ///
    /// # Parameters
    /// - `key`: The name of the header.
    /// - `value`: The value of the header.
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn insert_header<K: Into<HeaderName>>(self, key: K, value: impl Into<HeaderValue>) -> Self {
        self.update(|parts| {
            parts.headers.insert(key.into(), value);
            Ok(())
        })
    }

    /// Appends a value to an existing header in the request.
    ///
    /// If the header does not exist, it is created.
    ///
    /// # Parameters
    /// - `key`: The name of the header.
    /// - `value`: The value to append.
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn append_header<K: Into<HeaderName>>(self, key: K, value: impl Into<HeaderValue>) -> Self {
        self.update(|parts| {
            parts.headers.append(key.into(), value);
            Ok(())
        })
    }

    /// Builds the final `Request` with the provided body.
    ///
    /// # Parameters
    /// - `body`: The content to include in the request body.
    ///
    /// # Returns
    /// A `Request` with the method, version, URL, headers, and body.
    pub fn build<T>(self, body: T) -> Result<Request<T>, InvalidRequest> {
        let inner = self.inner?;
        let Parts {
            headers,
            method,
            version,
            uri,
        } = inner;

        Ok(Request {
            method,
            version,
            uri,
            headers,
            body,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
