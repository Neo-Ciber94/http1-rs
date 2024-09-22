use super::{
    headers::{HeaderName, HeaderValue, Headers},
    method::Method,
    uri::{PathAndQuery, Uri},
    version::Version,
};

#[derive(Debug)]
pub struct Request<T> {
    headers: Headers,
    method: Method,
    version: Version,
    url: Uri,
    body: T,
}

impl<T> Request<T> {
    pub fn new(method: Method, version: Version, url: Uri, body: T) -> Self {
        Request {
            body,
            method,
            version,
            url,
            headers: Headers::default(),
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    pub fn url(&self) -> &Uri {
        &self.url
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }
}

impl Request<()> {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new()
    }
}

pub struct RequestBuilder {
    headers: Headers,
    method: Method,
    version: Version,
    url: Uri,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            headers: Headers::new(),
            method: Method::GET,
            url: Uri::new(None, None, PathAndQuery::new("/".to_owned(), None, None)),
            version: Version::Http1_1,
        }
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    pub fn url(mut self, url: impl Into<Uri>) -> Self {
        self.url = url.into();
        self
    }

    pub fn url_mut(&mut self) -> &mut Uri {
        &mut self.url
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    pub fn insert_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.insert(key.into(), value);
        self
    }

    pub fn append_header<K: Into<HeaderName>>(
        mut self,
        key: K,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.headers.append(key.into(), value);
        self
    }

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
