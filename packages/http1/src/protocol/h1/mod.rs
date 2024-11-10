pub mod request;
pub mod response;

use std::io::ErrorKind;

use crate::{
    body::Body, handler::RequestHandler, method::Method, request::Request, server::Config,
};

use super::connection::{Connected, Connection};

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H, C>(handler: &H, config: &Config, conn: C) -> std::io::Result<()>
where
    H: RequestHandler + Send + Sync + 'static,
    C: Connection + Send + 'static,
{
    let mut write_conn = match conn.try_clone() {
        Ok(conn) => conn,
        Err(err) => {
            return Err(std::io::Error::other(err.to_string()));
        }
    };

    // Create the request object
    let mut request = request::read_request(conn, config)?;

    // Append any extra information to the request
    pre_process_request(&mut request, &write_conn, config);

    // Get the response from the handler
    let discard_body = request.method() == Method::HEAD;
    let response = handler.handle(request);

    // Write the response to the stream
    match response::write_response(response, &mut write_conn, discard_body, config) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => {
            log::debug!("Connection was aborted");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn pre_process_request<C>(request: &mut Request<Body>, conn: &C, config: &Config)
where
    C: Connection + Send + 'static,
{
    if config.include_conn_info {
        request
            .extensions_mut()
            .insert(Connected::from_connection(conn));
    }

    if config.include_server_info {
        request.extensions_mut().insert(config.clone());
    }
}

#[cfg(test)]
mod pipe {
    use std::convert::Infallible;
    use std::io::{self, Cursor, Read, Write};
    use std::sync::{Arc, Mutex};

    use crate::protocol::connection::Connection;

    #[derive(Clone)]
    struct Inner {
        read_buffer: Cursor<Vec<u8>>,
        write_buffer: Vec<u8>,
    }

    #[derive(Clone)]
    pub struct Pipe {
        inner: Arc<Mutex<Inner>>,
    }

    impl Pipe {
        pub fn new(bytes: impl Into<std::borrow::Cow<'static, [u8]>>) -> Self {
            let bytes = bytes.into();

            Self {
                inner: Arc::new(Mutex::new(Inner {
                    read_buffer: Cursor::new(bytes.into()),
                    write_buffer: vec![],
                })),
            }
        }

        pub fn into_writer(self) -> Vec<u8> {
            self.inner.lock().unwrap().write_buffer.clone()
        }
    }

    impl<'a> From<&'a str> for Pipe {
        fn from(value: &'a str) -> Self {
            Pipe::new(value.as_bytes().to_vec())
        }
    }

    impl From<String> for Pipe {
        fn from(value: String) -> Self {
            Pipe::from(value.as_str())
        }
    }

    impl Read for Pipe {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut inner = self.inner.lock().unwrap();
            inner.read_buffer.read(buf)
        }
    }

    impl Write for Pipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut inner = self.inner.lock().unwrap();
            inner.write_buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.lock().unwrap().write_buffer.flush()
        }
    }

    impl Connection for Pipe {
        type Err = Infallible;

        fn try_clone(&self) -> Result<Self, Self::Err> {
            Ok(Self {
                inner: Arc::clone(&self.inner),
            })
        }

        fn peer_addr(&self) -> Option<std::net::SocketAddr> {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{response::Response, server::Config, status::StatusCode};

    use super::{handle_incoming, pipe::Pipe};

    #[test]
    fn should_get_request_and_get_response() {
        let pipe = Pipe::from("GET / HTTP/1.1\nHost: localhost:3000");

        let config = Config {
            include_date_header: false,
            ..Default::default()
        };

        let handler = |_| Response::new(StatusCode::OK, "Hello World!".into());

        handle_incoming(&handler, &config, pipe.clone()).unwrap();

        let data = pipe.into_writer();
        let response_text = std::io::read_to_string(data.as_slice()).unwrap();

        assert_eq!(
            response_text,
            "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello World!"
        );
    }
}
