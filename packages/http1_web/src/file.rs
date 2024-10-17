use std::{
    io::{BufReader, ErrorKind},
    path::{Path, PathBuf},
    str::FromStr,
};

use http1::{body::Body, headers, request::Request, response::Response, status::StatusCode};

use crate::{handler::Handler, into_response::IntoResponse, mime::Mime};

/// Sends a file response.
pub struct ServeFile(PathBuf);

impl ServeFile {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut file_path = std::env::current_dir().expect("failed to get current directory");

        let path = path.as_ref();
        file_path.push(path);

        assert!(!path.is_absolute(), "file_path cannot be absolute {path:?}");
        assert!(file_path.is_file(), "path is not a file: {file_path:?}");

        ServeFile(file_path)
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

            // let reader = BufReader::new(file);
            // let body = Body::new(reader);
            let bytes = std::fs::read(file_path).unwrap();
            let length = bytes.len();
            println!("{file_path:?} - size: {}", bytes.len());
            let body = Body::new(bytes);


            Response::builder()
                .insert_header(headers::CONTENT_TYPE, mime.to_string())
                .insert_header(headers::CONTENT_LENGTH, length.to_string())
                .body(body)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            eprintln!("Error serving file: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
