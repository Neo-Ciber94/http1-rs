use std::fmt::Display;

use crate::{
    extensions::Extensions,
    headers::{InvalidHeaderName, InvalidHeaderValue},
};

use super::{
    headers::{HeaderName, HeaderValue, Headers},
    method::Method,
    uri::{
        path_query::PathAndQuery,
        uri::{InvalidUri, Uri},
    },
    version::Version,
};

/// Request parts.
#[derive(Debug)]
pub struct Parts {
    pub headers: Headers,
    pub method: Method,
    pub version: Version,
    pub uri: Uri,
    pub extensions: Extensions,
}

impl Default for Parts {
    fn default() -> Self {
        Self {
            headers: Default::default(),
            method: Method::GET,
            version: Default::default(),
            uri: Uri::new(None, None, PathAndQuery::new(String::from("/"), None, None)),
            extensions: Default::default(),
        }
    }
}

/// Represents an HTTP request.
///
/// This struct holds the HTTP method, version, URL, headers, and body of the request.
/// The body is generic, allowing flexibility in request content.
#[derive(Debug)]
pub struct Request<T> {
    body: T,
    parts: Parts,
}

impl<T> Request<T> {
    /// Creates a new `Request` with the specified method, version, URL, and body.
    ///
    /// # Parameters
    /// - `method`: The HTTP method of the request (e.g., `GET`, `POST`).
    /// - `uri`: The URL of the request.
    /// - `body`: The content of the request body.
    ///
    /// # Returns
    /// A new `Request` instance with empty headers.
    pub fn new(method: Method, uri: Uri, body: T) -> Self {
        Request {
            body,
            parts: Parts {
                method,
                uri,
                version: Version::Http1_1,
                headers: Headers::default(),
                extensions: Extensions::new(),
            },
        }
    }

    /// Creates a request from the given parts and body.
    pub fn from_parts(parts: Parts, body: T) -> Self {
        Request { body, parts }
    }

    /// Returns a reference to the HTTP method of the request.
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    /// Returns a mutable reference to the HTTP method of the request.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.parts.method
    }

    /// Returns a reference to the HTTP version of the request.
    pub fn version(&self) -> &Version {
        &self.parts.version
    }

    /// Returns a mutable reference to the HTTP version of the request.
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.parts.version
    }

    /// Returns a reference to the URL of the request.
    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }

    /// Returns a mutable reference to the URL of the request.
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.parts.uri
    }

    /// Returns a reference to the headers of the request.
    pub fn headers(&self) -> &Headers {
        &self.parts.headers
    }

    /// Returns a mutable reference to the headers of the request.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.parts.headers
    }

    /// Returns this request extensions.
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    /// Returns a mutable reference to the request extensions.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    /// Returns a reference to the body of the request.
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the body of the request.
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    /// Returns a reference to the request parts.
    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    /// Returns a mutable reference to the request parts.
    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    /// Returns the body
    pub fn into_body(self) -> T {
        self.body
    }

    /// Split this request into its body and rest of the parts.
    pub fn into_parts(self) -> (T, Parts) {
        let Self { body, parts } = self;
        (body, parts)
    }

    /// Separates the request and the body.
    pub fn drop_body(self) -> (Request<()>, T) {
        let Request { body, parts } = self;
        (Request::from_parts(parts, ()), body)
    }

    /// Maps this request body to other type.
    pub fn map_body<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        Request {
            body: f(self.body),
            parts: self.parts,
        }
    }
}

impl Request<()> {
    /// Returns a `Builder` for constructing a new `Request`.
    ///
    /// # Example
    /// ```
    /// use http1::{method::Method,request::Request};
    ///
    /// let request = Request::builder()
    ///     .method(Method::POST)
    ///     .uri("/submit")
    ///     .body("body content")
    ///     .unwrap();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
}

#[derive(Debug)]
pub enum InvalidRequest {
    InvalidUri(InvalidUri),
    InvalidHeaderName(InvalidHeaderName),
    InvalidHeaderValue(InvalidHeaderValue),
}

impl Display for InvalidRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidRequest::InvalidUri(invalid_uri) => write!(f, "{invalid_uri}"),
            InvalidRequest::InvalidHeaderName(invalid_header_name) => {
                write!(f, "invalid request, {invalid_header_name}")
            }
            InvalidRequest::InvalidHeaderValue(invalid_header_value) => {
                write!(f, "invalid request, {invalid_header_value}")
            }
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
            extensions: Default::default(),
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

    /// Returns a mutable reference to the extensions.
    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.inner.as_mut().ok().map(|parts| &mut parts.extensions)
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
    pub fn insert_header<K, V>(self, name: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<InvalidHeaderName>,
        V: TryInto<HeaderValue>,
        V::Error: Into<InvalidHeaderValue>,
    {
        self.update(|parts| {
            let key = name
                .try_into()
                .map_err(|err| InvalidRequest::InvalidHeaderName(err.into()))?;
            let value = value
                .try_into()
                .map_err(|err| InvalidRequest::InvalidHeaderValue(err.into()))?;

            parts.headers.insert(key, value);
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
    pub fn append_header<K, V>(self, name: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<InvalidHeaderName>,
        V: TryInto<HeaderValue>,
        V::Error: Into<InvalidHeaderValue>,
    {
        self.update(|parts| {
            let key = name
                .try_into()
                .map_err(|err| InvalidRequest::InvalidHeaderName(err.into()))?;
            let value = value
                .try_into()
                .map_err(|err| InvalidRequest::InvalidHeaderValue(err.into()))?;

            parts.headers.append(key, value);
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
    pub fn body<T>(self, body: T) -> Result<Request<T>, InvalidRequest> {
        let inner = self.inner?;
        let Parts {
            headers,
            method,
            version,
            uri,
            extensions,
        } = inner;

        Ok(Request {
            body,
            parts: Parts {
                method,
                version,
                uri,
                headers,
                extensions,
            },
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
