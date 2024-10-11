use http1::{
    body::Body,
    headers::{self, HeaderValue},
    status::StatusCode,
};

use crate::{from_request::FromRequest, into_response::IntoResponse};

pub struct FormData {
    boundary: String,
    body: Body,
}

struct FormDataField {
    name: String,
    bytes: Vec<u8>,
    filename: Option<String>,
}

pub struct FormDataFieldError;

impl FormData {
    pub fn new(boundary: impl Into<String>, body: impl Into<Body>) -> Self {
        FormData {
            boundary: boundary.into(),
            body: body.into(),
        }
    }

    pub fn next_field(&mut self) -> Result<Option<FormDataField>, FormDataFieldError> {
        todo!()
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

            if boundary.starts_with("\"") && boundary.ends_with("\"") {
                return Ok(boundary[1..(boundary.len())].to_owned());
            }

            Err(FormDataError::InvalidBoundary(boundary.to_owned()))
        }
        None => Err(FormDataError::BoundaryNoFound),
    }
}
