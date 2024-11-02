use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use crate::{
    body::{
        body_reader::{ChunkedBodyReader, FixedLengthBodyReader},
        Body,
    },
    headers::{self, HeaderName, Headers, CONTENT_LENGTH, TRANSFER_ENCODING},
    method::Method,
    request::Request,
    server::Config,
    uri::uri::Uri,
    version::Version,
};

pub fn read_request<R: Read + Send + 'static>(
    stream: R,
    config: &Config,
) -> std::io::Result<Request<Body>> {
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
    let body = read_request_body(reader, &headers, can_discard_body, config)?;

    // Set headers
    if let Some(req_headers) = builder.headers_mut() {
        req_headers.extend(headers);
    }

    let request = builder
        .body(body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    Ok(request)
}

fn read_request_body<R: Read + Send + 'static>(
    reader: BufReader<R>,
    headers: &Headers,
    can_discard_body: bool,
    config: &Config,
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
        Body::new(FixedLengthBodyReader::new(
            reader,
            Some(length),
            config.max_body_size,
        ))
    } else if let Some(encoding) = transfer_encoding {
        // Read body based on Chunked Transfer-Encoding
        if encoding == "chunked" {
            Body::new(ChunkedBodyReader::new(reader, config.max_body_size))
        } else {
            return Err(std::io::Error::other(format!(
                "Unknown transfer encoding: `{encoding}`"
            )));
        }
    } else {
        // Read until the connection closes
        Body::new(FixedLengthBodyReader::new(
            reader,
            None,
            config.max_body_size,
        ))
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
        .and_then(|s| crate::uri::url_encoding::decode(s).ok())
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

    // Special case for cookies because are separated by `;` instead of `,`
    let values = if name.eq_ignore_ascii_case(headers::COOKIE.as_str()) {
        rest.split(";")
            .map(|s| s.trim())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
    } else {
        rest.split(",")
            .map(|s| s.trim())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
    };

    Some((name, values))
}
