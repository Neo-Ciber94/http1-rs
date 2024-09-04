use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    body::Body,
    handler::RequestHandler,
    http::{
        headers::{HeaderName, Headers},
        method::Method,
        request::Request,
        response::Response,
        uri::Uri,
        version::Version,
    },
};

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Server { addr }
    }

    fn handle_incoming(
        handler: Arc<dyn RequestHandler + Send + Sync + 'static>,
        mut stream: TcpStream,
    ) {
        let request = read_request(&mut stream).expect("Failed to read request");
        let response = handler.handle(request);
        write_response(response, &mut stream).unwrap()
    }

    pub fn listen<H: RequestHandler + Send + Sync + 'static>(
        self,
        handler: H,
    ) -> std::io::Result<()> {
        let addr = self.addr;
        let listener = TcpListener::bind(&self.addr)?;
        let handler_mutex = Mutex::new(Arc::new(handler));

        println!("Listening on: {addr:?}");

        for connection in listener.incoming() {
            match connection {
                Ok(stream) => {
                    let lock = handler_mutex.lock().expect("Failed to acquire lock");
                    let request_handler = lock.clone();
                    std::thread::spawn(move || Self::handle_incoming(request_handler, stream));
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Request<Body>> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();

    // Read first line
    reader.read_line(&mut buf)?;

    let mut builder = Request::builder();

    let (method, url, version) = read_request_line(&buf)?;
    *builder.method_mut() = method;
    *builder.url_mut() = url;
    *builder.version_mut() = version;

    // Read headers
    let mut headers = Headers::new();
    buf.clear();

    while reader.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim();

        if line.is_empty() {
            break;
        }

        if let Some((key, values)) = read_header(line) {
            values.iter().for_each(|v| {
                headers.append(HeaderName::from_string(key.clone()), v);
            })
        }

        buf.clear();
    }

    Ok(builder.build(buf.into()))
}

fn read_request_line(buf: &str) -> std::io::Result<(Method, Uri, Version)> {
    let str = buf.trim();
    let mut parts = str.splitn(3, " ");

    let method = parts
        .next()
        .and_then(|x| Method::from_str(x).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse request method"))?;

    let url = parts
        .next()
        .and_then(|s| Uri::from_str(s).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse request url"))?;

    let version = parts
        .next()
        .and_then(|x| Version::from_str(x).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse http version"))?;

    Ok((method, url, version))
}

fn read_header(buf: &str) -> Option<(String, Vec<String>)> {
    let str = buf.trim();
    let (key, rest) = str.split_once(": ")?;
    let mut values = Vec::new();

    for value in rest.split(";").map(|s| s.trim()) {
        values.push(value.to_owned())
    }

    Some((key.to_owned(), values))
}

fn write_response(response: Response<Body>, stream: &mut TcpStream) -> std::io::Result<()> {
    let version = response.version();
    let (status, headers, body) = response.into_parts();
    let reason_phrase = status.reason_phrase().unwrap_or("");

    // 1. Write response line
    write!(stream, "{version} {status} {reason_phrase}\r\n")?;

    // 2. Write headers
    for (name, mut values) in headers {
        write!(stream, "{name}: ")?;

        if let Some(first_value) = values.next() {
            stream.write_all(first_value.as_bytes())?;
        }

        for value in values {
            stream.write_all(b", ")?;
            stream.write_all(value.as_bytes())?;
        }

        stream.write_all(b"\r\n")?;
    }

    // Headers end
    write!(stream, "\r\n")?;

    // 3. Write body
    match body {
        Body::Payload(mut data) => {
            stream.write_all(data.as_mut())?;
            stream.flush()?;
        }
        Body::Stream(recv) => {
            while let Ok(mut chunk) = recv.recv() {
                // Write the chunk size
                let size = chunk.len();
                write!(stream, "{size:X}\r\n")?;

                // Write the chunk data
                stream.write_all(chunk.as_mut())?;
                write!(stream, "\r\n")?;

                // Send
                stream.flush()?;
            }

            // Last chunk
            write!(stream, "0\r\n\r\n")?;
        }
    }

    Ok(())
}
