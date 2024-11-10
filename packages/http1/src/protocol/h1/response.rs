use std::{
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

use crate::{
    body::{body_writer::BodyWriter, http_body::HttpBody, Body},
    headers::{self, Headers},
    response::Response,
    server::Config,
    status::StatusCode,
    version::Version,
};
use datetime::DateTime;

pub fn write_response<W: Write>(
    response: Response<Body>,
    stream: &mut W,
    discard_body: bool,
    config: &Config,
) -> std::io::Result<()> {
    let version = response.version();
    let (status, headers, mut body, ..) = response.into_parts();
    let reason_phrase = status.reason_phrase().unwrap_or("");

    // 1. Write response line
    write!(stream, "{version} {status} {reason_phrase}\r\n")?;

    // 2. Write headers
    write_headers(headers, &body, stream, config)?;

    // 3. Write body
    if !discard_body {
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
        // If the response provided a content-length we trust it
        if !headers.contains_key(headers::CONTENT_LENGTH) {
            headers.insert(headers::CONTENT_LENGTH, content_length.to_string());
        }
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

pub fn read_response<R: Read>(reader: R) -> std::io::Result<Response<Body>> {
    let mut buf = String::new();
    let mut buf_reader = BufReader::new(reader);

    let mut response = Response::builder();

    // Read response line
    let (version, status) = read_response_line(&mut buf_reader, &mut buf)?;
    *response.version_mut() = version;
    *response.status_mut() = status;

    // Read headers, it will read all the headers until a empty line is found
    let headers = crate::protocol::h1::request::read_headers(&mut buf_reader, &mut buf)?;
    response.headers_mut().extend(headers);

    // Read body
    let body = read_response_body(&mut buf_reader)?;

    Ok(response.body(body))
}

fn read_response_line<R: Read>(
    reader: &mut BufReader<R>,
    mut buf: &mut String,
) -> std::io::Result<(Version, StatusCode)> {
    reader.read_line(&mut buf)?;

    if buf.is_empty() {
        return Err(std::io::Error::other("response was empty"));
    }

    let (version_str, status_code_str) = buf.split_once(" ").ok_or_else(|| {
        std::io::Error::other("failed to get response HTTP version and status code")
    })?;

    let version = Version::from_str(version_str).map_err(std::io::Error::other)?;

    // We ignore the status code phrase
    let status_code = status_code_str
        .split(" ")
        .next()
        .and_then(|s| u16::from_str(s).ok())
        .and_then(|s| StatusCode::try_from_status(s).ok())
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "Failed to parse status code from `{status_code_str}`"
            ))
        })?;

    Ok((version, status_code))
}

const BODY_READER_BUFFER_SIZE: usize = 4 * 1024; // 4kb

fn read_response_body<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<Body> {
    let (body, sender) = BodyWriter::<Vec<u8>>::new();

    let mut buf = vec![0; BODY_READER_BUFFER_SIZE].into_boxed_slice();

    loop {
        match reader.read(&mut buf)? {
            0 => break,
            n => {
                let chunk = buf[..n].to_vec();
                sender.send(chunk).map_err(std::io::Error::other)?;
            }
        }
    }

    Ok(Body::new(body))
}
