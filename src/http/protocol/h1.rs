use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::TcpStream,
    str::FromStr,
};

use crate::{
    body::{http_body::HttpBody, Body},
    common::date_time::DateTime,
    handler::RequestHandler,
    http::{
        headers::{self, HeaderName, Headers},
        method::Method,
        request::Request,
        response::Response,
        uri::Uri,
        version::Version,
    },
    server::ServerConfig,
};

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H: RequestHandler + Send + Sync + 'static>(
    handler: &H,
    config: &ServerConfig,
    mut stream: TcpStream,
) {
    let request = read_request(&mut stream).expect("Failed to read request");
    let response = handler.handle(request);
    match write_response(response, &mut stream, &config) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => {}
        Err(err) => eprintln!("{err}"),
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
            values.into_iter().for_each(|v| {
                headers.append(HeaderName::from_string(key.clone()), v);
            })
        }

        buf.clear();
    }

    // Read the body
    let body = buf.into();
    Ok(builder.build(body))
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

fn write_response(
    response: Response<Body>,
    stream: &mut TcpStream,
    config: &ServerConfig,
) -> std::io::Result<()> {
    let version = response.version();
    let (status, headers, mut body) = response.into_parts();
    let reason_phrase = status.reason_phrase().unwrap_or("");

    // 1. Write response line
    write!(stream, "{version} {status} {reason_phrase}\r\n")?;

    // 2. Write headers
    write_headers(headers, &body, stream, config)?;

    // 3. Write body
    loop {
        match body.read_next() {
            Ok(Some(bytes)) => {
                stream.write_all(&bytes)?;
                stream.flush()?;
            }
            Ok(None) => break,
            Err(err) => return Err(std::io::Error::other(err)),
        }
    }

    Ok(())
}

fn write_headers(
    mut headers: Headers,
    body: &Body,
    stream: &mut TcpStream,
    config: &ServerConfig,
) -> std::io::Result<()> {
    if config.include_date_header {
        headers.insert(headers::DATE, DateTime::now_utc().to_string());
    }

    if let Some(content_length) = body.size_hint() {
        headers.insert(headers::CONTENT_LENGTH, content_length.to_string());
    }

    for (name, mut values) in headers {
        write!(stream, "{name}: ")?;

        if let Some(first_value) = values.next() {
            stream.write_all(first_value.as_str().as_bytes())?;
        }

        for value in values {
            stream.write_all(b", ")?;
            stream.write_all(value.as_str().as_bytes())?;
        }

        stream.write_all(b"\r\n")?;
    }

    // Headers end
    write!(stream, "\r\n")?;

    Ok(())
}
