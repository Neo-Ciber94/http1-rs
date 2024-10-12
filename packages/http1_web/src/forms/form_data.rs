use std::io::{BufRead, BufReader};

use http1::{
    body::{body_reader::BodyReader, Body},
    headers::{self, HeaderValue},
    status::StatusCode,
};

use crate::{from_request::FromRequest, into_response::IntoResponse};

pub struct Field {
    name: String,
    bytes: Vec<u8>,
    filename: Option<String>,
}

#[derive(Debug)]
pub enum FieldError {
    MissingBoundary(String),
    IO(std::io::Error),
    Other(String),
    MissingContentDisposition,
    MissingName,
    MissingFilename,
}

pub struct FormData {
    reader: BufReader<BodyReader>,
    boundary: String,
    str_buf: String,
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
            str_buf: String::new(),
        }
    }

    fn read_newline(&mut self) -> Result<(), FieldError> {
        self.str_buf.clear();
        let buf = &mut self.str_buf;

        match self.reader.read_line(buf) {
            Ok(_) => {
                if buf != "\n\r" {
                    return Err(FieldError::Other(format!("expected new line")));
                }
            }
            Err(err) => return Err(FieldError::IO(err)),
        }

        Ok(())
    }

    fn read_field_boundary(&mut self) -> Result<(), FieldError> {
        self.str_buf.clear();
        let buf = &mut self.str_buf;

        match self.reader.read_line(buf) {
            Ok(_) => {
                if buf != &self.boundary {
                    return Err(FieldError::MissingBoundary(self.boundary.clone()));
                }
            }
            Err(err) => {
                return Err(FieldError::IO(err));
            }
        }

        Ok(())
    }

    fn read_content_disposition(&mut self) -> Result<ContentDisposition, FieldError> {
        self.str_buf.clear();
        let buf = &mut self.str_buf;

        match self.reader.read_line(buf) {
            Ok(_) => {
                let mut name: Result<String, FieldError> = Err(FieldError::MissingName);
                let mut filename: Result<Option<String>, FieldError> = Ok(None);

                let mut parts = buf.split(";").map(|s| s.trim());

                if Some("Content-Disposition: form-data") != parts.next() {
                    return Err(FieldError::MissingContentDisposition);
                }

                while let Some(s) = parts.next() {
                    match s {
                        _ if s.starts_with("name=") => {
                            name = get_quoted("\"", &s[5..])
                                .map(|s| s.to_owned())
                                .ok_or(FieldError::MissingName);
                        }
                        _ if s.starts_with("filename=") => {
                            filename = get_quoted("\"", &s[9..])
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
            Err(err) => {
                return Err(FieldError::IO(err));
            }
        }
    }

    pub fn next_field(&mut self) -> Result<Option<Field>, FieldError> {
        // Read the newline after the last field/boundary
        self.read_newline()?;

        // Read the boundary that starts the new field
        self.read_field_boundary()?;

        // Read the content disposition to get the field name and filename
        let ContentDisposition { name, filename } = self.read_content_disposition()?;

        // Read another newline after the headers
        self.read_newline()?;

        // Prepare buffer to store field data
        let mut bytes = Vec::new();
        let mut buf = &mut self.str_buf;

        // Read the field content until the next boundary
        loop {
            buf.clear();
            let bytes_read = self.reader.read_line(&mut buf).map_err(FieldError::IO)?;

            if bytes_read == 0 {
                return Err(FieldError::Other(
                    "Unexpected EOF while reading field data".into(),
                ));
            }

            // Check if the line is the boundary
            if buf.starts_with(&self.boundary) {
                break; // Boundary reached, stop reading the field data
            }

            // Append the read bytes to the field's data buffer
            bytes.extend_from_slice(buf.as_bytes());
        }

        // Return the field with name, filename, and data
        Ok(Some(Field {
            bytes,
            name,
            filename,
        }))
    }
}

const MULTIPART_FORM_DATA: &str = "multipart/form-data";

pub enum FormDataError {
    NoContentType,
    InvalidContentType(String),
    BoundaryNoFound,
    InvalidBoundary(String),
}

impl IntoResponse for FormDataError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
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

            match get_quoted("\"", boundary) {
                Some(s) => Ok(s.to_owned()),
                None => Err(FormDataError::InvalidBoundary(boundary.to_owned())),
            }
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
