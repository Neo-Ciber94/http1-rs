use std::{borrow::Cow, fmt::Display, net::TcpStream, time::Duration};

use serde::ser::Serialize;

use crate::{
    body::Body,
    error::BoxError,
    headers::{self, HeaderName, HeaderValue, Headers, InvalidHeaderName, InvalidHeaderValue},
    method::Method,
    request::{InvalidRequest, Request},
    response::Response,
    uri::{
        scheme::Scheme,
        uri::{InvalidUri, Uri},
    },
};

/// A request error.
const DEFAULT_USER_AGENT: &str = "rust";

#[derive(Debug)]
pub enum RequestError {
    InvalidRequest(InvalidRequest),
    IO(std::io::Error),
    FailedToConnect { addr: String, err: std::io::Error },
    Other(BoxError),
}

impl std::error::Error for RequestError {}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::InvalidRequest(error) => write!(f, "{error}"),
            RequestError::IO(error) => write!(f, "{error}"),
            RequestError::FailedToConnect { addr, err } => {
                write!(f, "failed to connect to `{addr}`: {err}")
            }
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

/// A http client.
pub struct Client {
    user_agent: Option<String>,
    default_headers: Headers,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl Client {
    /// Constructs a [`Client`].
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Returns a [`Client`] builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Returns the `User-Agent` for this client.
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    /// Returns the default headers for this client.
    pub fn default_headers(&self) -> &Headers {
        &self.default_headers
    }

    /// Returns a request builder.
    pub fn request<T>(&self, method: Method, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        RequestBuilder::new(self, method, url)
    }

    /// Returns a `GET` request builder.
    pub fn get<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::GET, url)
    }

    /// Returns a `POST` request builder.
    pub fn post<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::POST, url)
    }

    /// Returns a `PUT` request builder.
    pub fn put<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::PUT, url)
    }

    /// Returns a `PATCH` request builder.
    pub fn patch<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::PATCH, url)
    }

    /// Returns a `DELETE` request builder.
    pub fn delete<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::DELETE, url)
    }

    /// Returns a `OPTION` request builder.
    pub fn option<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::OPTIONS, url)
    }

    /// Returns a `HEAD` request builder.
    pub fn head<T>(&self, url: T) -> RequestBuilder
    where
        T: TryInto<Uri>,
        T::Error: Into<InvalidUri>,
    {
        self.request(Method::HEAD, url)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ClientBuilder(Client);

impl ClientBuilder {
    /// Constructs a new [`ClientBuilder`].
    pub fn new() -> Self {
        ClientBuilder(Client {
            user_agent: None,
            default_headers: Headers::new(),
            read_timeout: None,
            write_timeout: None,
        })
    }

    /// Sets the user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.0.user_agent = Some(user_agent.into());
        self
    }

    /// Append a default header.
    pub fn append_default_header(
        mut self,
        name: HeaderName,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.0.default_headers.append(name, value.into());
        self
    }

    /// Insert a default header.
    pub fn insert_default_header(
        mut self,
        name: HeaderName,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.0.default_headers.insert(name, value.into());
        self
    }

    /// Add a list of headers.
    pub fn default_headers(mut self, headers: Headers) -> Self {
        self.0.default_headers.extend(headers);
        self
    }

    /// Sets the read timeout.
    pub fn read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.0.read_timeout = timeout;
        self
    }

    /// Sets the write timeout.
    pub fn write_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.0.write_timeout = timeout;
        self
    }

    /// Builds the [`Client`].
    pub fn build(self) -> Client {
        self.0
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestBuilder<'a> {
    request: crate::request::Builder,
    client: &'a Client,
}

impl<'a> RequestBuilder<'a> {
    /// Constructs a new [`RequestBuilder`].
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

    /// Sets the request headers.
    pub fn headers(self, headers: Headers) -> Self {
        self.tap(|mut request| {
            if let Some(h) = request.headers_mut() {
                h.extend(headers);
            }

            request
        })
    }

    /// Append a header.
    pub fn append_header<K, V>(self, name: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<InvalidHeaderName>,
        V: TryInto<HeaderValue>,
        V::Error: Into<InvalidHeaderValue>,
    {
        self.tap(|request| request.append_header(name, value))
    }

    /// Insert a header.
    pub fn insert_header<K, V>(self, name: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<InvalidHeaderName>,
        V: TryInto<HeaderValue>,
        V::Error: Into<InvalidHeaderValue>,
    {
        self.tap(|request| request.insert_header(name, value))
    }

    /// Sends the request using the given `JSON` as body.
    pub fn json<T: Serialize>(self, json: &T) -> Result<Response<Body>, RequestError> {
        let json_bytes =
            serde::json::to_bytes(json).map_err(|err| RequestError::Other(err.into()))?;

        self.append_header(
            headers::CONTENT_TYPE,
            HeaderValue::from_static("application/json;charset-utf8"),
        )
        .send(json_bytes)
    }

    /// Sends a request with the given body.
    pub fn send(self, body: impl Into<Body>) -> Result<Response<Body>, RequestError> {
        let Self { request, client } = self;

        let user_agent = client
            .user_agent
            .as_deref()
            .map(|s| Cow::Owned(s.to_owned()))
            .unwrap_or_else(|| Cow::Borrowed(DEFAULT_USER_AGENT));

        let mut request = request
            .insert_header(headers::USER_AGENT, user_agent)
            .insert_header(headers::ACCEPT, "*/*")
            .body(body.into())?;

        let (host, port) = get_addr(&request)?;
        let addr = format!("{host}:{port}");

        request
            .headers_mut()
            .insert(headers::HOST, HeaderValue::from_string(host.clone()));
        request.headers_mut().extend(client.default_headers.clone());

        let mut stream =
            TcpStream::connect(&addr).map_err(|err| RequestError::FailedToConnect { addr, err })?;

        stream.set_write_timeout(client.write_timeout)?;
        stream.set_read_timeout(client.read_timeout)?;

        crate::protocol::h1::request::write_request(&mut stream, request)?;

        let response = crate::protocol::h1::response::read_response(stream)?;

        Ok(response)
    }
}

fn get_addr(request: &Request<Body>) -> Result<(String, u16), RequestError> {
    let authority = request
        .uri()
        .authority()
        .ok_or_else(|| RequestError::Other(String::from("missing authority").into()))?;

    let is_https = request.uri().scheme().is_some_and(|s| *s == Scheme::Https);
    let host = authority.host().to_owned();
    let port: u16 = authority.port().unwrap_or(if is_https { 443 } else { 80 });

    Ok((host, port))
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::channel, time::Duration};

    use crate::{
        body::{chunked_body::ChunkedBody, http_body::HttpBody, Body},
        headers::{self, HeaderValue},
        method::Method,
        response::Response,
        server::Server,
        status::StatusCode,
    };

    use super::Client;

    #[test]
    fn should_send_request_to_server_and_get_response() {
        let port = crate::common::find_open_port::find_open_port().unwrap();
        let addr = format!("0.0.0.0:{port}");

        let server = Server::new();
        let handle = server.handle();

        let (tx, rx) = channel();
        let (ready_tx, ready_rx) = channel();

        {
            std::thread::spawn(move || {
                server
                    .on_ready(move |addr| {
                        println!("server running on: {addr}");
                        ready_tx.send(()).unwrap();
                    })
                    .listen(addr.clone(), move |request| {
                        tx.send(request).unwrap();
                        Response::new(StatusCode::OK, "Bloom Into You".into())
                    })
                    .unwrap();
            });
        }

        // Wait for server to be ready
        ready_rx
            .recv()
            .unwrap_or_else(|_| panic!("Server failed to start"));

        let client = Client::new();
        let res = client
            .request(Method::POST, format!("http://127.0.0.1:{port}"))
            .send(Body::from("Yagate Kimi ni Naru"))
            .unwrap();

        // Assert server
        let mut req = rx.recv().unwrap();

        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            req.body_mut().read_all_bytes().unwrap(),
            b"Yagate Kimi ni Naru"
        );

        // Assert client
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.into_body().read_all_bytes().unwrap(), b"Bloom Into You");

        // Shutdown server
        handle.shutdown();
    }

    #[test]
    fn should_send_request_to_server_and_get_chunked_encoded_response() {
        let port = crate::common::find_open_port::find_open_port_in_range(3001..).unwrap();
        let addr = format!("0.0.0.0:{port}");

        let server = Server::new();
        let handle = server.handle();
        let (tx, rx) = channel();
        let (ready_tx, ready_rx) = channel();

        {
            std::thread::spawn(move || {
                server
                    .on_ready(move |addr| {
                        println!("server running on: {addr}");
                        ready_tx.send(()).unwrap();
                    })
                    .listen(addr.clone(), move |request| {
                        tx.send(request).unwrap();
                        let (body, sender) = ChunkedBody::new();

                        std::thread::spawn(move || {
                            for i in 1..=3 {
                                std::thread::sleep(Duration::from_millis(10));
                                sender.send(format!("Chunk {i}\n")).unwrap();
                            }
                        });

                        Response::builder()
                            .status(StatusCode::CREATED)
                            .append_header(
                                headers::TRANSFER_ENCODING,
                                HeaderValue::from_static("chunked"),
                            )
                            .body(body.into())
                    })
                    .unwrap();
            });
        }

        // Wait for server to be ready
        ready_rx
            .recv()
            .unwrap_or_else(|_| panic!("Server failed to start"));

        let client = Client::builder()
            .read_timeout(Some(Duration::from_millis(5_000)))
            .build();
        let res = client
            .request(
                Method::GET,
                format!("http://127.0.0.1:{port}/message?text=hello"),
            )
            .send(())
            .unwrap();

        // Assert server
        let mut req = rx.recv().unwrap();

        assert_eq!(req.method(), Method::GET);
        assert_eq!(req.body_mut().read_all_bytes().unwrap(), b"");

        // Assert client
        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(
            res.into_body().read_all_bytes().unwrap(),
            b"Chunk 1\nChunk 2\nChunk 3\n"
        );

        // Shutdown server
        handle.shutdown();
    }

    #[test]
    fn should_get_example_com() {
        let client = Client::new();
        let result = client.get("http://example.com").send(()).unwrap();
        let bytes = result.into_body().read_all_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("<h1>Example Domain</h1>"))
    }
}
