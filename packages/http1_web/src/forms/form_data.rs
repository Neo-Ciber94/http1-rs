use core::str;
use std::{
    fmt::Display,
    io::{BufRead, BufReader, Read},
    str::Utf8Error,
};

use http1::{
    body::{body_reader::BodyReader, Body},
    headers::{self, HeaderValue},
    status::StatusCode,
};

use crate::{from_request::FromRequest, into_response::IntoResponse};

#[derive(Debug, PartialEq, Eq)]
enum State {
    First,
    Next,
    Done,
}

#[derive(Debug)]
pub enum FieldError {
    MissingBoundary(String),
    IO(std::io::Error),
    Other(String),
    Utf8Error(Utf8Error),
    MissingContentDisposition,
    MissingName,
    MissingFilename,
}

impl std::error::Error for FieldError {}

impl Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldError::MissingBoundary(boundary) => write!(f, "Missing boundary: {}", boundary),
            FieldError::IO(error) => write!(f, "{error}"),
            FieldError::Other(msg) => write!(f, "{msg}"),
            FieldError::MissingContentDisposition => {
                write!(f, "Missing content disposition header")
            }
            FieldError::Utf8Error(err) => write!(f, "{err}"),
            FieldError::MissingName => write!(f, "Missing 'name' field in content disposition"),
            FieldError::MissingFilename => {
                write!(f, "Missing 'filename' field in content disposition")
            }
        }
    }
}

pub struct FormData {
    reader: BufReader<BodyReader>,
    boundary: String,
    bytes_buf: Vec<u8>,
    state: State,
}

struct ContentDisposition {
    name: String,
    filename: Option<String>,
}

impl FormData {
    pub fn new(boundary: impl Into<String>, body: impl Into<Body>) -> Self {
        let reader = BodyReader::new(body.into());
        let buf_reader = BufReader::new(reader);
        let boundary = format!("--{}", boundary.into());

        FormData {
            boundary,
            reader: buf_reader,
            bytes_buf: Vec::new(),
            state: State::First,
        }
    }

    fn parse_newline(&mut self) -> Result<(), FieldError> {
        read_line(&mut self.reader, &mut self.bytes_buf)?;

        if !self.bytes_buf.is_empty() {
            return Err(FieldError::Other(format!("expected new line")));
        }

        Ok(())
    }

    fn parse_boundary(&mut self) -> Result<(), FieldError> {
        read_line(&mut self.reader, &mut self.bytes_buf)?;

        if &self.bytes_buf != &self.boundary.as_bytes() {
            return Err(FieldError::MissingBoundary(self.boundary.clone()));
        }

        Ok(())
    }

    fn parse_content_disposition(&mut self) -> Result<ContentDisposition, FieldError> {
        read_line(&mut self.reader, &mut self.bytes_buf)?;

        let mut name: Result<String, FieldError> = Err(FieldError::MissingName);
        let mut filename: Result<Option<String>, FieldError> = Ok(None);

        let mut parts = self.bytes_buf.split(|b| *b == b';').map(|s| s.trim_ascii());

        if Some("Content-Disposition: form-data".as_bytes()) != parts.next() {
            return Err(FieldError::MissingContentDisposition);
        }

        while let Some(s) = parts.next() {
            match s {
                _ if s.starts_with("name=".as_bytes()) => {
                    let text = str::from_utf8(s).map_err(FieldError::Utf8Error)?;
                    name = get_quoted("\"", &text[5..])
                        .map(|s| s.to_owned())
                        .ok_or(FieldError::MissingName);
                }
                _ if s.starts_with("filename=".as_bytes()) => {
                    let text = str::from_utf8(s).map_err(FieldError::Utf8Error)?;
                    filename = get_quoted("\"", &text[9..])
                        .map(|s| Some(s.to_owned()))
                        .ok_or(FieldError::MissingFilename);
                }
                _ => {
                    // ignore
                }
            }
        }

        Ok(ContentDisposition {
            name: name?,
            filename: filename?,
        })
    }

    fn field_reader<'a>(&'a mut self) -> FieldReader<'a> {
        self.bytes_buf.clear();
        FieldReader {
            form_data: self,
            byte_buf: Vec::new(),
        }
    }

    fn next_bytes(&mut self, buf: &mut Vec<u8>) -> Result<usize, FieldError> {
        let read_bytes = read_line(&mut self.reader, buf)?;

        let s = String::from_utf8_lossy(buf);
        dbg!(s);

        // Check if is boundary end or a field delimiter
        if buf.starts_with(self.boundary.as_bytes()) {
            println!("boundary!!");
            let boundary_len = self.boundary.len();
            let rest = &buf[boundary_len..];

            if rest == b"--" {
                self.state = State::Done;
            }

            buf.clear();

            Ok(0)
        } else {
            Ok(read_bytes)
        }
    }

    pub fn next_field<'a>(&'a mut self) -> Result<Option<Field<'a>>, FieldError> {
        if self.state == State::Done {
            return Ok(None);
        }

        if self.state == State::First {
            // Read the newline after the last field/boundary
            self.parse_newline()?;
            self.state = State::Next;
        }

        // Read the boundary that starts the new field
        self.parse_boundary()?;

        // Read the content disposition to get the field name and filename
        let ContentDisposition { name, filename } = self.parse_content_disposition()?;

        // Read another newline after the headers
        self.parse_newline()?;

        Ok(Some(Field {
            name,
            filename,
            form_data: self,
        }))
    }
}

fn read_line(reader: &mut BufReader<BodyReader>, buf: &mut Vec<u8>) -> Result<usize, FieldError> {
    buf.clear();

    match reader.read_until(b'\n', buf) {
        Ok(n) => {
            // Remove '\n'
            buf.pop();

            // Remove '\r'
            if buf.ends_with(b"\r") {
                buf.pop();
            }

            Ok(n)
        }
        Err(err) => return Err(FieldError::IO(err)),
    }
}

pub struct FieldReader<'a> {
    byte_buf: Vec<u8>,
    form_data: &'a mut FormData,
}

impl<'a> Read for FieldReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut pos = 0;
        let chunk = &mut self.byte_buf;

        while pos < buf.len() {
            if chunk.is_empty() {
                let read_bytes = self
                    .form_data
                    .next_bytes(chunk)
                    .map_err(std::io::Error::other)?;

                if read_bytes == 0 {
                    break;
                }
            }

            let left = buf.len() - pos;
            let len = left.min(chunk.len());

            for (idx, byte) in chunk.drain(..len).enumerate() {
                buf[pos + idx] = byte;
            }

            pos += len;
        }

        Ok(pos)
    }
}

pub struct Field<'a> {
    name: String,
    filename: Option<String>,
    form_data: &'a mut FormData,
}

impl<'a> Field<'a> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn bytes(self) -> Result<Vec<u8>, FieldError> {
        let mut bytes = Vec::new();
        self.reader()
            .read_to_end(&mut bytes)
            .map_err(FieldError::IO)?;

        Ok(bytes)
    }

    pub fn text(self) -> Result<String, FieldError> {
        let mut buf = String::new();
        self.reader()
            .read_to_string(&mut buf)
            .map_err(FieldError::IO)?;

        Ok(buf)
    }

    pub fn reader(self) -> FieldReader<'a> {
        self.form_data.field_reader()
    }
}

const MULTIPART_FORM_DATA: &str = "multipart/form-data";

#[derive(Debug)]
pub enum FormDataError {
    NoContentType,
    InvalidContentType(String),
    BoundaryNoFound,
    InvalidBoundary(String),
}

impl IntoResponse for FormDataError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        match self {
            FormDataError::NoContentType => {
                eprintln!("Missing `{MULTIPART_FORM_DATA}` content type")
            }
            FormDataError::InvalidContentType(s) => {
                eprintln!("Invalid content type: `{s}` expected `{MULTIPART_FORM_DATA}`")
            }
            FormDataError::BoundaryNoFound => eprintln!("Missing form boundary"),
            FormDataError::InvalidBoundary(s) => eprintln!("Invalid form boundary: `{s}`"),
        }

        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl FromRequest for FormData {
    type Rejection = FormDataError;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        let content_type = headers
            .get(headers::CONTENT_TYPE)
            .ok_or(FormDataError::NoContentType)?;

        let boundary = get_multipart_and_boundary(content_type)?;
        let body = req.into_body();
        Ok(FormData::new(boundary, body))
    }
}

fn get_multipart_and_boundary(value: &HeaderValue) -> Result<String, FormDataError> {
    let mut s = value.as_str().split(";");

    match s.next() {
        Some(mime) => {
            if mime != MULTIPART_FORM_DATA {
                return Err(FormDataError::InvalidContentType(mime.to_owned()));
            }
        }
        None => return Err(FormDataError::NoContentType),
    };

    match s.next() {
        Some(boundary_str) => {
            let (_, boundary) = boundary_str
                .split_once("boundary=")
                .ok_or_else(|| FormDataError::BoundaryNoFound)?;

            Ok(boundary.to_owned())
        }
        None => Err(FormDataError::BoundaryNoFound),
    }
}

fn get_quoted<'a>(quote: &'a str, value: &'a str) -> Option<&'a str> {
    if value.len() <= quote.len() {
        return None;
    }

    if value.starts_with(quote) && value.ends_with(quote) {
        let len = quote.len();
        return Some(&value[len..(value.len() - len)]);
    }

    return None;
}

#[cfg(test)]
mod tests {
    use http1::{body::Body, headers, request::Request};

    use crate::from_request::FromRequest;

    use super::FormData;

    #[test]
    fn read_form_data() {
        let boundary = "x-my-boundary";

        let mut s = String::new();
        s.push_str("\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(&format!(
            "Content-Disposition: form-data; name=\"text_field\"\r\n\r\n"
        ));
        s.push_str("This is the text content\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(&format!(
            "Content-Disposition: form-data; name=\"file_field\"; filename=\"file.txt\"\r\n"
        ));
        s.push_str("This is the content of the file\r\n");
        s.push_str(&format!("--{boundary}--\r\n"));

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!("multipart/form-data;boundary={boundary}"),
            )
            .body(Body::new(s))
            .unwrap();

        let mut form_data = FormData::from_request(req).unwrap();

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "text_field");
            assert_eq!(field.filename(), None);
            assert_eq!(field.text().unwrap(), "This is the text content");
        }

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "file_field");
            assert_eq!(field.filename(), Some("file.txt"));
            assert_eq!(field.text().unwrap(), "This is the content of the file");
        }
    }
}
