use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    body::Body,
    handler::RequestHandler,
    http::{HeaderName, Headers, Method, Request, Response, Url, Version},
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

fn read_request_line(buf: &str) -> std::io::Result<(Method, Url, Version)> {
    let str = buf.trim();
    let mut parts = str.splitn(3, " ");

    let method = parts
        .next()
        .and_then(|x| Method::from_str(x).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse request method"))?;

    let url = parts
        .next()
        .map(|s| Url(s.to_owned()))
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
    let (status, headers, body) = response.into_parts();
    let status_text = "Ok";

    // Write response line
    write!(stream, "HTTP/1.1 {status} {status_text}\r\n")?;

    // Write headers
    let keys = headers.keys();
    for name in keys {
        let values = headers.get_all(name);
        let line = values
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .join(", ");

        write!(stream, "{name}: {line}\r\n")?;
    }

    write!(stream, "\r\n")?;

    // Write body

    match body {
        Body::Payload(mut data) => {
            stream.write_all(data.as_mut())?;
            stream.flush()?;
        }
        Body::Stream(recv) => {
            while let Ok(mut bytes) = recv.recv() {
                stream.write_all(bytes.as_mut())?
            }
        }
    }

    Ok(())
}
