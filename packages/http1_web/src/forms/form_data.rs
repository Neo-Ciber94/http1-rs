use core::str;
use std::{
    fmt::{Debug, Display},
    io::Read,
    path::PathBuf,
    str::Utf8Error,
};

use http1::{
    body::{body_reader::BodyReader, Body},
    error::BoxError,
    extensions::Extensions,
    headers::{self, HeaderValue},
    request::Request,
    status::StatusCode,
};

use crate::{from_request::FromRequest, IntoResponse};

use super::{
    form_field::{Disk, FormField, Memory, TempDisk},
    stream_reader::StreamReader,
};

#[derive(Debug, PartialEq, Eq)]
enum State {
    /// Needs to read first field
    First,
    /// Reading any next fields.
    Next,
    /// Reached end boundary.
    Done,
}

#[derive(Debug)]
pub enum FieldError {
    MissingBoundary(String),
    MissingContentType,
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
            FieldError::MissingContentType => write!(f, "Missing file content-type"),
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

#[derive(PartialEq, Eq)]
enum ReadLineMode {
    None,
    Limited,
}

pub(crate) const MAX_HEADER_LENGTH: usize = 1024; // 1kb
pub(crate) const PARSE_BUFFER_SIZE: usize = 8 * 1024; // 8kn

/// Configuration for parsing `multipart/form-data`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDataConfig {
    /// Max allowed length for the headers.
    pub max_header_length: usize,

    /// Max allowed length for the entire form data body.
    pub max_body_size: usize,

    /// Size of the buffer used to parse the form data.
    pub buffer_size: usize,
}

impl FormDataConfig {
    pub fn max_header_length(mut self, max_header_length: usize) -> Self {
        self.max_header_length = max_header_length;
        self
    }

    pub fn max_body_size(mut self, max_body_size: usize) -> Self {
        self.max_body_size = max_body_size;
        self
    }

    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

impl Default for FormDataConfig {
    fn default() -> Self {
        Self {
            max_header_length: MAX_HEADER_LENGTH,
            max_body_size: http1::constants::DEFAULT_MAX_BODY_SIZE,
            buffer_size: PARSE_BUFFER_SIZE,
        }
    }
}

/// Represents multipart/form-data stream.
///
/// We parse the file according to: https://datatracker.ietf.org/doc/html/rfc7578
pub struct FormData {
    reader: StreamReader<BodyReader>,
    boundary: String,
    state: State,
    max_header_length: usize,
}

impl Debug for FormData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormData").finish()
    }
}

struct ContentDisposition {
    name: String,
    filename: Option<String>,
}

impl FormData {
    pub fn new(boundary: impl Into<String>, body: impl Into<Body>) -> Self {
        Self::with_config(boundary, body, Default::default())
    }

    pub fn with_config(
        boundary: impl Into<String>,
        body: impl Into<Body>,
        config: FormDataConfig,
    ) -> Self {
        let body_reader = BodyReader::new(body.into());
        let boundary = format!("--{}", boundary.into());
        let max_header_length = config.max_header_length;
        let reader = StreamReader::with_buffer_size_and_read_limit(
            body_reader,
            config.buffer_size,
            config.max_body_size,
        );

        FormData {
            boundary,
            reader,
            state: State::First,
            max_header_length,
        }
    }

    fn read_line(&mut self, mode: ReadLineMode) -> Result<Vec<u8>, FieldError> {
        let limit = if mode == ReadLineMode::Limited {
            Some(self.max_header_length)
        } else {
            None
        };

        let mut line = self
            .reader
            .read_line_with_limit(limit)
            .map_err(FieldError::IO)?;

        if line.ends_with(b"\r\n") {
            line.pop();
            line.pop();
        } else if line.ends_with(b"\n") {
            line.pop();
        }

        Ok(line)
    }

    fn parse_newline(&mut self) -> Result<(), FieldError> {
        let bytes = self.read_line(ReadLineMode::None)?;

        if !bytes.is_empty() {
            return Err(FieldError::Other(String::from("expected new line")));
        }

        Ok(())
    }

    fn parse_boundary(&mut self) -> Result<(), FieldError> {
        let line = self.read_line(ReadLineMode::None)?;

        if line != self.boundary.as_bytes() {
            return Err(FieldError::MissingBoundary(self.boundary.clone()));
        }

        Ok(())
    }

    fn parse_content_disposition(&mut self) -> Result<ContentDisposition, FieldError> {
        let bytes = self.read_line(ReadLineMode::Limited)?;
        let mut name: Result<String, FieldError> = Err(FieldError::MissingName);
        let mut filename: Result<Option<String>, FieldError> = Ok(None);
        let mut parts = bytes.split(|b| *b == b';').map(|s| s.trim_ascii());

        if Some("Content-Disposition: form-data".as_bytes()) != parts.next() {
            return Err(FieldError::MissingContentDisposition);
        }

        for s in parts {
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

    fn parse_content_type(&mut self) -> Result<String, FieldError> {
        let bytes = self.read_line(ReadLineMode::Limited)?;

        if !bytes.starts_with(b"Content-Type:") {
            return Err(FieldError::MissingContentType);
        }

        let rest = &bytes[b"Content-Type:".len()..].trim_ascii();
        let content_type = std::str::from_utf8(rest).map_err(FieldError::Utf8Error)?;
        Ok(content_type.to_owned())
    }

    fn field_reader(&mut self) -> FieldReader<'_> {
        FieldReader {
            form_data: self,
            chunk: Vec::new(),
            eof: false,
        }
    }

    fn next_chunk(&mut self, buf: &mut Vec<u8>) -> Result<bool, FieldError> {
        let boundary_str = format!("\r\n{}", self.boundary);

        let (found, mut bytes) = self
            .reader
            .read_until_sequence(boundary_str.as_bytes())
            .map_err(FieldError::IO)?;

        // No more bytes to read
        if bytes.is_empty() {
            return Ok(true);
        }

        // Boundary no found, continue reading
        if !found {
            buf.extend(bytes);
            return Ok(false);
        }

        // Boundary found, check if is the field end or form data end
        if found && bytes.ends_with(boundary_str.as_bytes()) {
            // Remove the boundary
            bytes.truncate(bytes.len() - boundary_str.len());

            // Check if this is the end of the form data
            let next_bytes = self.reader.peek(2).map_err(FieldError::IO)?;

            if next_bytes == b"--" {
                self.state = State::Done;
                self.reader.read_exact(2).map_err(FieldError::IO)?;
            }

            // Ensure after boundary there is a line ending `\r\n`
            let boundary_line_endings = self.reader.peek(2).map_err(FieldError::IO)?;

            if boundary_line_endings != b"\r\n" {
                return Err(FieldError::Other(String::from(
                    "expected line endings `\\r\\n` after boundary",
                )));
            }

            // Remove boundary line endings
            self.reader.read_exact(2).map_err(FieldError::IO)?;

            buf.extend(bytes);
            Ok(true)
        } else {
            // If is done it should contain the boundary
            Err(FieldError::Other(String::from("End boundary no found")))
        }
    }

    pub fn next_field(&mut self) -> Result<Option<Field<'_>>, FieldError> {
        if self.state == State::Done {
            return Ok(None);
        }

        if self.state == State::First {
            // We parse the first boundary, next boundaries will be consumed by the `Field`
            self.parse_boundary()?;
            self.state = State::Next;
        }

        // Read the content disposition to get the field name and filename
        let ContentDisposition { name, filename } = self.parse_content_disposition()?;

        // If it's a file parse the content-type
        let content_type = if filename.is_some() {
            Some(self.parse_content_type()?)
        } else {
            None
        };

        // Read another newline after the headers
        self.parse_newline()?;

        Ok(Some(Field {
            name,
            filename,
            content_type,
            form_data: self,
        }))
    }
}

pub struct FieldReader<'a> {
    chunk: Vec<u8>,
    form_data: &'a mut FormData,
    eof: bool,
}

impl<'a> FieldReader<'a> {
    fn read_chunk(&mut self) -> std::io::Result<()> {
        if self.eof {
            return Ok(());
        }

        match self.form_data.next_chunk(&mut self.chunk) {
            Ok(finished) => {
                if finished {
                    self.eof = true;
                }

                Ok(())
            }
            Err(err) => Err(std::io::Error::other(err)),
        }
    }
}

impl<'a> Read for FieldReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if self.chunk.is_empty() {
            self.read_chunk()?;
        }

        let mut read = 0;

        while read < buf.len() && !self.chunk.is_empty() {
            let chunk = &mut self.chunk;
            let remaining = buf.len() - read;
            let len = remaining.min(chunk.len());

            let src = &chunk[..len];
            let dst = &mut buf[read..(read + len)];
            dst.copy_from_slice(src);
            chunk.drain(..len);

            read += len;

            // Read next chunk
            if self.chunk.is_empty() {
                self.read_chunk()?;
            }
        }

        Ok(read)
    }
}

/// Represents a reference to a field in a form data stream.
pub struct Field<'a> {
    pub(crate) name: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
    form_data: &'a mut FormData,
}

impl<'a> Field<'a> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    pub fn bytes(self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.reader().read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn text(self) -> std::io::Result<String> {
        let mut buf = String::new();
        self.reader().read_to_string(&mut buf)?;
        Ok(buf)
    }

    pub fn reader(self) -> FieldReader<'a> {
        self.form_data.field_reader()
    }

    pub fn to_memory(self) -> std::io::Result<FormField<Memory>> {
        FormField::memory(self)
    }

    pub fn to_disk(self, file_path: impl Into<PathBuf>) -> std::io::Result<FormField<Disk>> {
        FormField::disk(self, file_path)
    }

    pub fn to_temp_file(self) -> std::io::Result<FormField<TempDisk>> {
        FormField::temp(self)
    }
}

impl Debug for Field<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("filename", &self.filename)
            .field("content_type", &self.content_type)
            .field("form_data", &self.form_data)
            .finish()
    }
}

const MULTIPART_FORM_DATA: &str = "multipart/form-data";

#[derive(Debug)]
pub enum FormDataError {
    NoContentType,
    InvalidContentType(String),
    BoundaryNoFound,
    InvalidBoundary(String),
    Other(BoxError),
}

impl Display for FormDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormDataError::NoContentType => {
                write!(f, "Missing `{MULTIPART_FORM_DATA}` content type")
            }
            FormDataError::InvalidContentType(s) => {
                write!(
                    f,
                    "Invalid content type: `{s}` expected `{MULTIPART_FORM_DATA}`"
                )
            }
            FormDataError::BoundaryNoFound => write!(f, "Missing form boundary"),
            FormDataError::InvalidBoundary(s) => write!(f, "Invalid form boundary: `{s}`"),
            FormDataError::Other(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for FormDataError {}

impl IntoResponse for FormDataError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl FromRequest for FormData {
    type Rejection = FormDataError;

    fn from_request(
        req: &Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        fn get_form_data_config(exts: &Extensions) -> FormDataConfig {
            match exts.get::<crate::state::State<FormDataConfig>>() {
                Some(crate::state::State(config)) => config.clone(),
                None => {
                    let mut config = FormDataConfig::default();
                    if let Some(max_body_size) = exts
                        .get::<http1::server::Config>()
                        .and_then(|b| b.max_body_size)
                    {
                        config.max_body_size = max_body_size;
                    }

                    config
                }
            }
        }

        let config = get_form_data_config(extensions);
        let headers = req.headers();
        let content_type = headers
            .get(headers::CONTENT_TYPE)
            .ok_or(FormDataError::NoContentType)?;

        let boundary = get_multipart_and_boundary(content_type)?;
        let body = payload.take().unwrap_or_default();
        Ok(FormData::with_config(boundary, body, config))
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

    None
}

#[cfg(test)]
enum TestFormField {
    Text {
        name: &'static str,
        value: &'static str,
    },
    File {
        name: &'static str,
        filename: &'static str,
        content_type: &'static str,
        value: std::borrow::Cow<'static, [u8]>,
    },
}

#[cfg(test)]
struct TestForm {
    boundary: &'static str,
    fields: Vec<TestFormField>,
}

#[cfg(test)]
impl TestForm {
    pub fn new(boundary: &'static str) -> Self {
        TestForm {
            boundary,
            fields: vec![],
        }
    }

    pub fn text(mut self, name: &'static str, value: &'static str) -> Self {
        self.fields.push(TestFormField::Text { name, value });
        self
    }

    pub fn file(
        mut self,
        name: &'static str,
        filename: &'static str,
        content_type: &'static str,
        value: std::borrow::Cow<'static, [u8]>,
    ) -> Self {
        self.fields.push(TestFormField::File {
            name,
            filename,
            content_type,
            value,
        });
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let TestForm { boundary, fields } = self;
        let mut form_data = vec![];

        // start
        form_data.extend_from_slice(format!("--{boundary}\r\n").as_bytes());

        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                form_data.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            }

            match field {
                TestFormField::Text { name, value } => {
                    let content_disposition =
                        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n");
                    form_data.extend_from_slice(content_disposition.as_bytes());
                    form_data.extend_from_slice(value.as_bytes());
                    form_data.extend_from_slice(b"\r\n");
                }
                TestFormField::File {
                    name,
                    filename,
                    content_type,
                    value,
                } => {
                    let content_disposition = format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n");
                    form_data.extend_from_slice(content_disposition.as_bytes());

                    let content_type = format!("Content-Type: {content_type}\r\n\r\n");
                    form_data.extend_from_slice(content_type.as_bytes());
                    form_data.extend_from_slice(value);
                    form_data.extend_from_slice(b"\r\n");
                }
            }
        }

        // end
        form_data.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        form_data
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use http1::{body::Body, extensions::Extensions, headers, payload::Payload, request::Request};

    use crate::{forms::form_data::TestForm, from_request::FromRequest};

    use super::{FormData, FormDataConfig};

    #[test]
    fn should_read_text_and_then_file() {
        let boundary = "x-my-boundary";

        let mut s = String::new();
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str("Content-Disposition: form-data; name=\"text_field\"\r\n\r\n");
        s.push_str("This is the text content\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(
            "Content-Disposition: form-data; name=\"file_field\"; filename=\"file.txt\"\r\n",
        );
        s.push_str("Content-Type: text/plain\r\n\r\n");
        s.push_str("This is the content of the file\r\n");
        s.push_str(&format!("--{boundary}--\r\n"));

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!("multipart/form-data;boundary={boundary}"),
            )
            .body(())
            .unwrap();

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(s)),
        )
        .unwrap();

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

        {
            let field = form_data.next_field().unwrap();
            assert!(field.is_none());
        }
    }

    #[test]
    fn should_read_file_and_then_text() {
        let boundary = "x-my-boundary";

        let mut s = String::new();
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(
            "Content-Disposition: form-data; name=\"file_field\"; filename=\"file.txt\"\r\n",
        );
        s.push_str("Content-Type: text/plain\r\n\r\n");
        s.push_str("This is the content of the file\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str("Content-Disposition: form-data; name=\"text_field\"\r\n\r\n");
        s.push_str("This is the text content\r\n");

        s.push_str(&format!("--{boundary}--\r\n"));

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!("multipart/form-data;boundary={boundary}"),
            )
            .body(())
            .unwrap();

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(s)),
        )
        .unwrap();

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "file_field");
            assert_eq!(field.filename(), Some("file.txt"));
            assert_eq!(field.text().unwrap(), "This is the content of the file");
        }

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "text_field");
            assert_eq!(field.filename(), None);
            assert_eq!(field.text().unwrap(), "This is the text content");
        }

        {
            let field = form_data.next_field().unwrap();
            assert!(field.is_none());
        }
    }

    #[test]
    fn should_read_multiple_form_data_fields() {
        let boundary = "WebKitFormBoundaryDtbT5UpPj83kllfw";

        let mut s = String::new();
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(
            "Content-Disposition: form-data; name=\"uploads[]\"; filename=\"somebinary.dat\"\r\n",
        );
        s.push_str("Content-Type: application/octet-stream\r\n\r\n");
        s.push_str("some binary data...maybe the bits of a image..\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str(
            "Content-Disposition: form-data; name=\"uploads[]\"; filename=\"sometext.txt\"\r\n",
        );
        s.push_str("Content-Type: text/plain\r\n\r\n");
        s.push_str("hello how are you\r\n");
        s.push_str(&format!("--{boundary}\r\n"));
        s.push_str("Content-Disposition: form-data; name=\"input1\"\r\n\r\n");
        s.push_str("value1\r\n");
        s.push_str(&format!("--{boundary}--\r\n"));

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(())
            .unwrap();

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(s)),
        )
        .unwrap();

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "uploads[]");
            assert_eq!(field.filename(), Some("somebinary.dat"));
            assert_eq!(field.content_type().unwrap(), "application/octet-stream");
            assert_eq!(
                field.text().unwrap(),
                "some binary data...maybe the bits of a image.."
            );
        }

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "uploads[]");
            assert_eq!(field.filename(), Some("sometext.txt"));
            assert_eq!(field.content_type().unwrap(), "text/plain");
            assert_eq!(field.text().unwrap(), "hello how are you");
        }

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "input1");
            assert_eq!(field.filename(), None);
            assert_eq!(field.text().unwrap(), "value1");
        }

        {
            let field = form_data.next_field().unwrap();
            assert!(field.is_none());
        }
    }

    #[test]
    fn should_read_byte_buffer() {
        let mut binary_data = Vec::new();

        for i in 0..256 {
            binary_data.push((i % 256) as u8);
        }

        let boundary = "x-my-boundary";
        let mut content = Vec::new();

        // First field (binary)
        content.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        content.extend_from_slice(
            "Content-Disposition: form-data; name=\"binary_field\"; filename=\"binary.bin\"\r\n"
                .as_bytes(),
        );
        content.extend_from_slice("Content-Type: application/octet-stream\r\n\r\n".as_bytes());
        content.extend_from_slice(&binary_data);
        content.extend_from_slice(b"\r\n");

        content.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!("multipart/form-data;boundary={boundary}"),
            )
            .body(())
            .unwrap();

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(content)),
        )
        .unwrap();

        {
            let field = form_data.next_field().unwrap().unwrap();
            assert_eq!(field.name(), "binary_field");
            assert_eq!(field.filename(), Some("binary.bin"));
            let received_data = field.bytes().unwrap();
            assert_eq!(received_data, binary_data, "different bytes");
        }

        {
            let field = form_data.next_field().unwrap();
            assert!(field.is_none());
        }
    }

    #[test]
    fn should_read_byte_buffer_with_fixed_newline() {
        let mut binary_data = std::iter::successors(Some(0), |n| Some(n % 255))
            .take(512)
            .collect::<Vec<u8>>();

        binary_data.insert(binary_data.len() / 2, b'\n');

        let form = TestForm::new("my-form-data").file(
            "binary_field",
            "binary.bin",
            "application/octet-stream",
            binary_data.clone().into(),
        );

        let form_data = form.to_bytes();

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!(
                    "multipart/form-data;boundary={boundary}",
                    boundary = form.boundary
                ),
            )
            .body(())
            .unwrap();

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(form_data)),
        )
        .unwrap();

        let field = form_data.next_field().unwrap().unwrap();
        assert_eq!(field.name(), "binary_field");
        assert_eq!(field.filename(), Some("binary.bin"));
        assert_eq!(field.bytes().unwrap(), binary_data);
    }

    #[test]
    fn should_fail_to_read_form_data_with_body_size_limit() {
        let binary_data = std::iter::successors(Some(0), |n| Some(n % 255))
            .take(512)
            .collect::<Vec<u8>>();

        let form = TestForm::new("my-form-data").file(
            "binary_field",
            "binary.bin",
            "application/octet-stream",
            binary_data.into(),
        );

        let form_data = form.to_bytes();

        let req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!(
                    "multipart/form-data;boundary={boundary}",
                    boundary = form.boundary
                ),
            )
            .body(())
            .unwrap();

        let mut app_state = Extensions::default();
        app_state.insert(crate::state::State(FormDataConfig {
            max_body_size: 200,
            buffer_size: 16,
            ..Default::default()
        }));

        let mut extensions = Extensions::new();
        extensions.extend(app_state.clone());

        let mut form_data = FormData::from_request(
            &req,
            &mut extensions,
            &mut Payload::Data(Body::new(form_data)),
        )
        .unwrap();

        let field = form_data.next_field().unwrap().unwrap();
        assert_eq!(field.name(), "binary_field");
        assert_eq!(field.filename(), Some("binary.bin"));
        assert!(field.bytes().is_err());
    }

    #[test]
    fn should_fail_to_read_form_data_header_size_limit() {
        let form = TestForm::new("my-form-data").text("short", "hello").file(
            "this is a large binary field name",
            "this_is_a_large_file_name_for_the_binary_data_send_to_the_form_object.json",
            "application/json;charset=utf8",
            "Hello".as_bytes().into(),
        );

        let form_data = form.to_bytes();

        let mut req = Request::builder()
            .append_header(
                headers::CONTENT_TYPE,
                format!(
                    "multipart/form-data;boundary={boundary}",
                    boundary = form.boundary
                ),
            )
            .body(())
            .unwrap();

        let mut app_state = Extensions::default();
        app_state.insert(FormDataConfig {
            max_header_length: 100,
            ..Default::default()
        });

        req.extensions_mut().insert(Arc::new(app_state));

        let mut form_data = FormData::from_request(
            &req,
            &mut Extensions::new(),
            &mut Payload::Data(Body::new(form_data)),
        )
        .unwrap();

        let field1 = form_data.next_field();
        assert!(field1.is_ok());

        let field2 = form_data.next_field();
        assert!(field2.is_err());
    }
}
