use super::{
    headers::{HeaderName, HeaderValue, Headers},
    method::Method,
    uri::{PathAndQuery, Uri},
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
    url: Uri,
    body: T,
}

impl<T> Request<T> {
    /// Creates a new `Request` with the specified method, version, URL, and body.
    ///
    /// # Parameters
    /// - `method`: The HTTP method of the request (e.g., `GET`, `POST`).
    /// - `version`: The HTTP version used (e.g., `HTTP/1.1`).
    /// - `url`: The URL of the request.
    /// - `body`: The content of the request body.
    ///
    /// # Returns
    /// A new `Request` instance with empty headers.
    pub fn new(method: Method, version: Version, url: Uri, body: T) -> Self {
        Request {
            body,
            method,
            version,
            url,
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
    pub fn url(&self) -> &Uri {
        &self.url
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
    /// let request = Request::builder()
    ///     .method(Method::POST)
    ///     .url("/submit")
    ///     .build("body content");
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// A builder for constructing `Request` objects.
///
/// The `Builder` allows setting the HTTP method, version, URL, and headers before
/// building the final `Request` with a body.
pub struct Builder {
    headers: Headers,
    method: Method,
    version: Version,
    url: Uri,
}

impl Builder {
    /// Creates a new `Builder` with default values:
    /// - `GET` method
    /// - `HTTP/1.1` version
    /// - URL set to "/"
    /// - Empty headers
    pub fn new() -> Self {
        Builder {
            headers: Headers::new(),
            method: Method::GET,
            url: Uri::new(None, None, PathAndQuery::new("/".to_owned(), None, None)),
            version: Version::Http1_1,
        }
    }

    /// Sets the HTTP version of the request.
    ///
    /// # Parameters
    /// - `version`: The HTTP version to set (e.g., `Version::Http1_1`).
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Returns a mutable reference to the HTTP version.
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    /// Sets the HTTP method of the request.
    ///
    /// # Parameters
    /// - `method`: The HTTP method to set (e.g., `Method::POST`).
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Returns a mutable reference to the HTTP method.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    /// Sets the URL of the request.
    ///
    /// # Parameters
    /// - `url`: The URL to set.
    ///
    /// # Returns
    /// The updated `Builder`.
    pub fn url(mut self, url: impl Into<Uri>) -> Self {
        self.url = url.into();
        self
    }

    /// Returns a mutable reference to the URL.
    pub fn url_mut(&mut self) -> &mut Uri {
        &mut self.url
    }

    /// Returns a reference to the headers being set in the `Builder`.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a mutable reference to the headers being set in the `Builder`.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
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
    pub fn insert_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.insert(key.into(), value);
        self
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
    pub fn append_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.append(key.into(), value);
        self
    }

    /// Builds the final `Request` with the provided body.
    ///
    /// # Parameters
    /// - `body`: The content to include in the request body.
    ///
    /// # Returns
    /// A `Request` with the method, version, URL, headers, and body.
    pub fn build<T>(self, body: T) -> Request<T> {
        let Self {
            headers,
            method,
            version,
            url,
        } = self;

        Request {
            method,
            version,
            url,
            headers,
            body,
        }
    }
}
