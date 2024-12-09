use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use crate::{
    body::{
        body_reader::{ChunkedBodyReader, FixedLengthBodyReader},
        http_body::HttpBody,
        Body,
    },
    headers::{self, HeaderName, HeaderValue, Headers, CONTENT_LENGTH, TRANSFER_ENCODING},
    method::Method,
    request::{Parts, Request},
    server::Config,
    uri::uri::Uri,
    version::Version,
};

fn read_line<R: Read>(buf: &mut String, mut reader: R) -> std::io::Result<()> {
    let mut bytes = Vec::new();
    let mut byte_buf = [0; 1];
    let mut prev = None;

    loop {
        match reader.read(&mut byte_buf)? {
            0 => break,
            _ => {
                let c = byte_buf[0];
                bytes.push(c);

                if c == b'\n' && prev == Some(b'\r') {
                    bytes.pop();
                    bytes.pop();
                    break;
                }

                prev = Some(c);
            }
        }
    }

    let s = std::str::from_utf8(&bytes).map_err(std::io::Error::other)?;
    buf.push_str(s);

    Ok(())
}

pub fn read_request<R: Read + Send + 'static>(
    stream: R,
    config: &Config,
) -> std::io::Result<Request<Body>> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();

    // Read first line
    read_line(&mut buf, &mut reader)?;

    let mut builder = Request::builder();

    let (method, url, version) = read_request_line(&buf)?;
    let can_discard_body = method == Method::GET || method == Method::HEAD;

    *builder.method_mut().unwrap() = method;
    *builder.uri_mut().unwrap() = url;
    *builder.version_mut().unwrap() = version;

    // Read headers
    buf.clear();
    let headers = read_headers(&mut reader, &mut buf)?;

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

    if !str.is_ascii() {
        return Err(std::io::Error::other(format!(
            "invalid request line, expected ascii string but was: {str:?}"
        )));
    }

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

pub(crate) fn read_headers<R: Read>(
    reader: &mut BufReader<R>,
    buf: &mut String,
) -> std::io::Result<Headers> {
    buf.clear();

    let mut headers = Headers::new();

    loop {
        let read = reader.read_line(buf)?;
        let line = buf.trim();

        if line.is_empty() || read == 0 {
            break;
        }

        if let Some((name, values)) = parse_header_line(line) {
            values
                .into_iter()
                .try_for_each::<_, Result<(), std::io::Error>>(|v| {
                    let key =
                        HeaderName::try_from(name.to_owned()).map_err(std::io::Error::other)?;
                    let value = HeaderValue::try_from(v).map_err(std::io::Error::other)?;
                    headers.append(key, value);
                    Ok(())
                })?;
        }

        buf.clear();
    }

    Ok(headers)
}

fn parse_header_line(buf: &str) -> Option<(&str, Vec<String>)> {
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

pub fn write_request<W: std::io::Write>(
    mut writer: W,
    request: Request<Body>,
) -> std::io::Result<()> {
    let (
        mut body,
        Parts {
            uri,
            method,
            version,
            mut headers,
            ..
        },
    ) = request.into_parts();

    if !headers.contains_key(headers::CONTENT_LENGTH) {
        if let Some(size) = body.size_hint() {
            headers.append(headers::CONTENT_LENGTH, HeaderValue::from(size));
        }
    }

    // Write request line: <method> <path> <version>
    let path_query = uri.path_and_query();
    let line = format!("{method} {path_query} {version}\r\n");
    writer.write_all(line.as_bytes())?;

    // Write headers
    for (name, values) in headers {
        writer.write_all(name.as_str().as_bytes())?;
        writer.write_all(b": ")?;

        for (pos, val) in values.enumerate() {
            if pos > 0 {
                writer.write_all(b", ")?;
            }

            writer.write_all(val.as_str().as_bytes())?;
        }

        writer.write_all(b"\r\n")?;
    }

    // Write body
    writer.write_all(b"\r\n")?;

    while let Some(chunk) = body.read_next().map_err(std::io::Error::other)? {
        writer.write_all(&chunk)?;
    }

    Ok(())
}
