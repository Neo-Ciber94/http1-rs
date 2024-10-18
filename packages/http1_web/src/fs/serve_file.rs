use std::{
    io::{BufReader, ErrorKind},
    path::{Path, PathBuf},
};

use http1::{body::Body, headers, request::Request, response::Response, status::StatusCode};

use crate::{handler::Handler, into_response::IntoResponse, mime::Mime};

#[derive(Debug)]
pub enum InvalidFile {
    CurrentDirectoryNotFound,
    FileNotFound(PathBuf),
}

/// Sends a file response.
#[derive(Debug)]
pub struct ServeFile(PathBuf);

impl ServeFile {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, InvalidFile> {
        let cwd = std::env::current_dir().map_err(|_| InvalidFile::CurrentDirectoryNotFound)?;
        let file_path = cwd.join(path);

        if !file_path.is_file() {
            return Err(InvalidFile::FileNotFound(file_path));
        }

        Ok(ServeFile(file_path))
    }
}

impl Handler<Request<Body>> for ServeFile {
    type Output = Response<Body>;

    fn call(&self, _: Request<Body>) -> Self::Output {
        create_file_response(&self.0)
    }
}

impl IntoResponse for ServeFile {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        create_file_response(&self.0)
    }
}

fn create_file_response(file_path: &Path) -> Response<Body> {
    match std::fs::File::open(file_path) {
        Ok(file) => {
            let mime = file_path
                .file_name()
                .and_then(|x| x.to_str())
                .and_then(|x| Mime::guess_mime(x).ok())
                .unwrap_or(Mime::APPLICATION_OCTET_STREAM);

            let reader = BufReader::new(file);
            let body = Body::new(reader);

            Response::builder()
                .insert_header(headers::CONTENT_TYPE, mime.to_string())
                .body(body)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            log::error!("Error serving file: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
