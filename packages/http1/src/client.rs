use std::{fmt::Display, net::TcpStream};

use serde::ser::Serialize;

use crate::{
    body::Body,
    error::BoxError,
    headers::{self, HeaderName, HeaderValue, Headers},
    method::Method,
    request::{InvalidRequest, Request},
    response::Response,
    uri::uri::{InvalidUri, Uri},
};

const DEFAULT_USER_AGENT: &str = "rust";

#[derive(Debug)]
pub enum RequestError {
    InvalidRequest(InvalidRequest),
    IO(std::io::Error),
    Other(BoxError),
}

impl std::error::Error for RequestError {}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::InvalidRequest(error) => write!(f, "{error}"),
            RequestError::IO(error) => write!(f, "{error}"),
            RequestError::Other(error) => write!(f, "{error}"),
        }
    }
}

impl From<InvalidRequest> for RequestError {
    fn from(value: InvalidRequest) -> Self {
        RequestError::InvalidRequest(value)
    }
}

impl From<std::io::Error> for RequestError {
    fn from(value: std::io::Error) -> Self {
        RequestError::IO(value)
    }
}

pub struct Client {
    user_agent: Option<String>,
    default_headers: Headers,
}

impl Client {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn request<T>(&self, method: Method, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        RequestBuilder::new(self, method, url)
    }

    pub fn get<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::GET, url)
    }

    pub fn post<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::POST, url)
    }

    pub fn put<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::PUT, url)
    }

    pub fn patch<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::PATCH, url)
    }

    pub fn delete<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::DELETE, url)
    }

    pub fn option<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::OPTIONS, url)
    }

    pub fn head<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::HEAD, url)
    }
}

pub struct ClientBuilder(Client);

impl ClientBuilder {
    pub fn new() -> Self {
        ClientBuilder(Client {
            user_agent: None,
            default_headers: Headers::new(),
        })
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.0.user_agent = Some(user_agent.into());
        self
    }

    pub fn append_default_header(
        mut self,
        name: HeaderName,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.0.default_headers.append(name, value.into());
        self
    }

    pub fn insert_default_header(
        mut self,
        name: HeaderName,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.0.default_headers.insert(name, value.into());
        self
    }

    pub fn default_headers(mut self, headers: Headers) -> Self {
        self.0.default_headers.extend(headers);
        self
    }

    pub fn build(self) -> Client {
        self.0
    }
}

pub struct RequestBuilder<'a> {
    request: crate::request::Builder,
    client: &'a Client,
}

impl<'a> RequestBuilder<'a> {
    pub fn new<T>(client: &'a Client, method: Method, url: T) -> Self
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        let request = Request::builder().method(method).uri(url);
        RequestBuilder { request, client }
    }

    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(crate::request::Builder) -> crate::request::Builder,
    {
        RequestBuilder {
            request: f(self.request),
            client: self.client,
        }
    }

    pub fn uri<T>(self, uri: T) -> Self
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.tap(|request| request.uri(uri))
    }

    pub fn method(self, method: Method) -> Self {
        self.tap(|request| request.method(method))
    }

    pub fn headers(self, headers: Headers) -> Self {
        self.tap(|mut request| {
            if let Some(h) = request.headers_mut() {
                h.extend(headers);
            }

            request
        })
    }

    pub fn append_header(self, name: HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.tap(|mut request| {
            if let Some(h) = request.headers_mut() {
                h.append(name, value);
            }

            request
        })
    }

    pub fn insert_header(self, name: HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.tap(|mut request| {
            if let Some(h) = request.headers_mut() {
                h.insert(name, value);
            }

            request
        })
    }

    pub fn json<T: Serialize>(self, json: &T) -> Result<Response<Body>, RequestError> {
        let json_bytes =
            serde::json::to_bytes(json).map_err(|err| RequestError::Other(err.into()))?;

        self.append_header(headers::CONTENT_TYPE, "application/json;charset-utf8")
            .send(json_bytes)
    }

    pub fn send(self, body: impl Into<Body>) -> Result<Response<Body>, RequestError> {
        let Self { request, client } = self;
        let mut request = request.body(body.into())?;

        let user_agent = client
            .user_agent
            .as_deref()
            .unwrap_or(DEFAULT_USER_AGENT)
            .to_string();

        request
            .headers_mut()
            .append(headers::USER_AGENT, user_agent);

        request.headers_mut().extend(client.default_headers.clone());

        let url = request.uri().to_string();
        let mut stream = TcpStream::connect(url)?;

        crate::protocol::h1::request::write_request(&mut stream, request)?;

        let response = crate::protocol::h1::response::read_response(&mut stream)?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::find_open_port::find_open_port, response::Response, server::Server,
        status::StatusCode,
    };

    #[test]
    fn should_send_request_to_server_and_get_response() {
        let port = find_open_port().unwrap();
        let addr: (&str, u16) = ("127.0.0.1", port);

        let server = Server::new(addr);
        let handle = server.handle();

        std::thread::spawn(move || {
            server
                .start(|_| Response::new(StatusCode::OK, "Hello World!".into()))
                .unwrap();
        });
    }
}
