mod request;
mod response;

use std::{
    io::ErrorKind,
    sync::{Arc, Mutex},
};

use crate::{handler::RequestHandler, server::Config};

use super::connection::Connection;

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H, C>(handler: &H, config: &Config, conn: C) -> std::io::Result<()>
where
    H: RequestHandler + Send + Sync + 'static,
    C: Connection + Send + 'static,
{
    let mut writer = match conn.try_clone() {
        Ok(conn) => conn,
        Err(err) => {
            return Err(std::io::Error::other(err.to_string()));
        }
    };

    let request = request::read_request(conn, config)?;
    let response = handler.handle(request);

    match response::write_response(response, &mut writer, config) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => {
            log::debug!("Connection was aborted");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod pipe {
    use std::convert::Infallible;
    use std::io::{self, Cursor, Read, Write};
    use std::sync::{Arc, Mutex};

    use crate::protocol::connection::Connection;

    #[derive(Clone)]
    pub struct Pipe {
        buffer: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    impl Pipe {
        pub fn new(bytes: impl Into<std::borrow::Cow<'static, [u8]>>) -> Self {
            let bytes = bytes.into();

            Self {
                buffer: Arc::new(Mutex::new(Cursor::new(bytes.into()))),
            }
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
            let mut buffer = self.buffer.lock().unwrap();
            buffer.read(buf)
        }
    }

    impl Write for Pipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            dbg!(String::from_utf8_lossy(buf));
            let mut buffer = self.buffer.lock().unwrap();
            buffer.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.flush()
        }
    }

    impl Connection for Pipe {
        type Err = Infallible;

        fn try_clone(&self) -> Result<Self, Self::Err> {
            Ok(Self {
                buffer: Arc::clone(&self.buffer),
            })
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{response::Response, server::Config, status::StatusCode};

    use super::{handle_incoming, pipe::Pipe};

    trait StrExt {
        fn trim_margin(&self, margin: &str) -> String;
    }

    impl<'a> StrExt for &'a str {
        fn trim_margin(&self, margin: &str) -> String {
            self.trim()
                .lines()
                .map(|line| line.trim_start().trim_start_matches(margin))
                .map(|line| format!("{line}\n"))
                .collect()
        }
    }

    #[test]
    fn should_get_request_and_get_response() {
        let request_body = r#"
        | GET / HTTP/1.1
        | Host: localhost:3000
        "#
        .trim_margin("| ");

        dbg!(&request_body);

        let mut pipe = Pipe::from(request_body);

        let config = Config {
            include_date_header: false,
            ..Default::default()
        };

        let handler = |_| Response::new(StatusCode::OK, "Hello World!".into());

        handle_incoming(&handler, &config, pipe.clone()).unwrap();

        let response_text = std::io::read_to_string(&mut pipe).unwrap();
        assert_eq!(
            response_text,
            r#"
        | HTTP/1.1 200 OK
        | 
        | Hello World!
        "#
            .trim_margin("| ")
        );
    }
}
