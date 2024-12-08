pub(crate) mod io;
pub mod request;
pub mod response;

use std::io::ErrorKind;

use crate::{
    body::Body, handler::RequestHandler, headers, method::Method, request::Request, server::Config,
};

use super::{
    connection::{Connected, Connection},
    upgrade::{PendingUpgrade, Upgrade},
};

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H>(handler: &H, config: &Config, conn: Connection) -> std::io::Result<()>
where
    H: RequestHandler + Send + Sync + 'static,
{
    let mut write_conn = match conn.try_clone() {
        Some(conn) => conn,
        None => {
            return Err(std::io::Error::other("failed to clone connection"));
        }
    };

    // Create the request object
    println!("reading request");
    let mut request = request::read_request(conn, config)?;

    println!("request was read");

    // If the connection can be upgraded, we create a pending upgrade
    let can_be_upgraded = is_upgrade_request(&request);
    let pending_upgrade = if can_be_upgraded {
        let (sender, pending) = PendingUpgrade::new();
        let conn = write_conn
            .try_clone()
            .expect("failed to clone connection stream for upgrade connection");

        request.extensions_mut().insert(pending);
        Some((sender, conn))
    } else {
        None
    };

    // Append any extra information to the request
    pre_process_request(&mut request, &write_conn, config);

    // Get the response from the handler
    let discard_body = request.method() == Method::HEAD;
    let response = handler.handle(request);

    // Write the response to the stream
    match response::write_response(response, &mut write_conn, discard_body, config) {
        Ok(_) => {
            // If the connection can be upgrade, notify after write the response
            if let Some((notifier, conn)) = pending_upgrade {
                let upgrade = Upgrade::new(conn);
                notifier.notify(upgrade);
            }

            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => {
            log::debug!("Connection was aborted");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn pre_process_request(request: &mut Request<Body>, conn: &Connection, config: &Config) {
    if config.include_conn_info {
        request
            .extensions_mut()
            .insert(Connected::from_connection(conn));
    }

    if config.include_server_info {
        request.extensions_mut().insert(config.clone());
    }
}

fn is_upgrade_request(req: &Request<Body>) -> bool {
    req.headers()
        .get(headers::CONNECTION)
        .is_some_and(|conn| conn.as_str() == "Upgrade")
}

#[cfg(test)]
mod tests {
    use crate::protocol::connection::Connection;
    use crate::{response::Response, server::Config, status::StatusCode};

    use super::handle_incoming;
    use std::io::{self, Cursor, Read, Write};
    use std::sync::{Arc, Mutex};

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

    #[test]
    fn should_get_request_and_get_response() {
        let pipe = Pipe::from("GET / HTTP/1.1\nHost: localhost:3000");

        let config = Config {
            include_date_header: false,
            ..Default::default()
        };

        let handler = |_| Response::new(StatusCode::OK, "Hello World!".into());
        let conn = Connection::from_io(pipe.clone());
        handle_incoming(&handler, &config, conn).unwrap();

        let data = pipe.into_writer();
        let response_text = std::io::read_to_string(data.as_slice()).unwrap();

        assert_eq!(
            response_text,
            "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello World!"
        );
    }
}
