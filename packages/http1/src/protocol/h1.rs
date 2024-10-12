use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::TcpStream,
    str::FromStr,
};

use crate::{
    body::{
        body_reader::{BodyReader, ChunkedBodyReader},
        http_body::HttpBody,
        Body,
    },
    common::date_time::DateTime,
    handler::RequestHandler,
    headers::{self, HeaderName, Headers, CONTENT_LENGTH, TRANSFER_ENCODING},
    method::Method,
    request::Request,
    response::Response,
    server::Config,
    uri::uri::Uri,
    version::Version,
};

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H>(handler: &H, config: &Config, stream: TcpStream) -> std::io::Result<()>
where
    H: RequestHandler + Send + Sync + 'static,
{
    let mut writer = stream.try_clone()?;
    let request = read_request(stream)?;
    let response = handler.handle(request);

    match write_response(response, &mut writer, config) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => Ok(()),
        Err(err) => Err(err),
    }
}

fn read_request(stream: TcpStream) -> std::io::Result<Request<Body>> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();

    // Read first line
    reader.read_line(&mut buf)?;

    let mut builder = Request::builder();

    let (method, url, version) = read_request_line(&buf)?;
    let can_discard_body = method == Method::GET || method == Method::HEAD;

    *builder.method_mut().unwrap() = method;
    *builder.uri_mut().unwrap() = url;
    *builder.version_mut().unwrap() = version;

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
                headers.append(HeaderName::from(key), v);
            })
        }

        buf.clear();
    }

    // Read the body
    let body = read_request_body(reader, &headers, can_discard_body)?;

    // Set headers
    if let Some(req_headers) = builder.headers_mut() {
        req_headers.extend(headers);
    }

    let request = builder
        .build(body.into())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    Ok(request)
}

fn read_request_body(
    reader: BufReader<TcpStream>,
    headers: &Headers,
    can_discard_body: bool,
) -> std::io::Result<Body> {
    let content_length = headers
        .get(CONTENT_LENGTH)
        .and_then(|x| x.as_str().parse().ok());

    let transfer_encoding = headers.get(TRANSFER_ENCODING).map(|x| x.as_str());

    if can_discard_body && transfer_encoding.is_none() && content_length.is_none() {
        return Ok(Body::empty());
    }

    let body = if let Some(length) = content_length {
        // Read body based on Content-Length
        Body::new(BodyReader::new(reader, Some(length)))
    } else if let Some(encoding) = transfer_encoding {
        // Read body based on Chunked Transfer-Encoding
        if encoding == "chunked" {
            Body::new(ChunkedBodyReader::new(reader))
        } else {
            return Err(std::io::Error::other(format!(
                "Unknown transfer encoding: {encoding}"
            )));
        }
    } else {
        // Read until the connection closes
        Body::new(BodyReader::new(reader, None))
    };

    Ok(body)
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
        .and_then(|s| crate::uri::convert::decode_uri_component(s).ok())
        .and_then(|s| Uri::from_str(&s).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse request url"))?;

    let version = parts
        .next()
        .and_then(|x| Version::from_str(x).ok())
        .ok_or_else(|| std::io::Error::other("Failed to parse http version"))?;

    Ok((method, url, version))
}

fn read_header(buf: &str) -> Option<(&str, Vec<String>)> {
    let str = buf.trim();
    let (name, rest) = str.split_once(": ")?;
    let values = rest
        .split(",")
        .map(|s| s.trim())
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    Some((name, values))
}

fn write_response<W: Write>(
    response: Response<Body>,
    stream: &mut W,
    config: &Config,
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

fn write_headers<W: Write>(
    mut headers: Headers,
    body: &Body,
    stream: &mut W,
    config: &Config,
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
