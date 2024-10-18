use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use http1::{
    body::Body, common::date_time::DateTime, headers, request::Request, response::Response,
    status::StatusCode,
};

use crate::{
    error_response::{ErrorResponse, ErrorStatusCode},
    handler::Handler,
    html::{self, element::Element},
    into_response::IntoResponse,
    mime::Mime,
};

#[derive(Debug)]
pub struct ServeDir {
    from: String,
    to: PathBuf,
    index_html: bool,
    list_directory: bool,
}

impl ServeDir {
    pub fn new(from_route: impl Into<String>, to_dir: impl Into<PathBuf>) -> Self {
        let cwd = std::env::current_dir().expect("unable to get current directory");
        let to = cwd.join(to_dir.into());
        let from = from_route.into();

        assert!(from.starts_with("/"), "from route should starts with `/`");
        assert!(!from.ends_with("/"), "from route cannot ends with `/`");
        assert!(to.is_dir(), "`{to:?}` is not a directory");

        ServeDir {
            from,
            to,
            index_html: true,
            list_directory: false,
        }
    }

    pub fn index_html(mut self, index_html: bool) -> Self {
        self.index_html = index_html;
        self
    }

    pub fn list_directory(mut self, list_directory: bool) -> Self {
        self.list_directory = list_directory;
        self
    }
}

impl Handler<Request<Body>> for ServeDir {
    type Output = Response<Body>;

    fn call(&self, req: Request<Body>) -> Self::Output {
        let path = req.uri().path_and_query().path();
        let from = &self.from;

        println!("request path: {path}");
        if !path.starts_with(from) {
            return StatusCode::NOT_FOUND.into_response();
        }

        let rest = &path[from.len()..];
        let file_exists = Path::new(rest).exists();
        let has_extension = rest.split("/").last().filter(|x| x.contains(".")).is_some();
        let base_dir = &self.to;

        // If the request its to a path without extension and there is not a file there, we try to get the /index.html for that path
        let file_path = if !file_exists && !has_extension && self.index_html {
            let index = format!("{rest}/index.html");
            let index_file = base_dir.join(index.trim_start_matches("/"));

            if index_file.exists() {
                index_file
            } else {
                base_dir.join(rest.trim_start_matches("/"))
            }
        } else {
            base_dir.join(rest.trim_start_matches("/"))
        };

        println!("file_path: {file_path:?} - base_dir: {base_dir:?} - path: {path} - rest: {rest}");

        if !file_path.exists() {
            return StatusCode::NOT_FOUND.into_response();
        }

        let mime = file_path
            .extension()
            .and_then(|x| x.to_str())
            .and_then(|ext| Mime::from_extension(ext).ok())
            .unwrap_or(Mime::APPLICATION_OCTET_STREAM);

        if file_path.is_dir() {
            if self.list_directory {
                return list_directory_html(&self.from, rest, &file_path).into_response();
            } else {
                return StatusCode::NOT_FOUND.into_response();
            }
        }

        match File::open(&file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Response::builder()
                    .append_header(headers::CONTENT_TYPE, mime.to_string())
                    .body(Body::new(reader))
            }
            Err(err) => {
                eprintln!("Failed to open file: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

fn list_directory_html(
    base_dir: &str,
    route: &str,
    dir: &Path,
) -> Result<Option<Element>, ErrorResponse> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| {
        eprintln!("Failed to list directory: {err}");
        ErrorResponse::new(ErrorStatusCode::InternalServerError, ())
    })?;

    let dir_name = format!("{base_dir}{}", if route.is_empty() { "/" } else { route });

    Ok(html::html(|| {
        html::head(|| {
            html::title(dir_name.clone());
        });

        html::body(|| {
            html::h1(format!("Directory: {dir_name}"));

            html::table(|| {
                html::thead(|| {
                    html::tr(|| {
                        html::th("File Name");
                        html::th("Modification Date");
                        html::th("File Size (Bytes)");
                    });
                });

                html::tbody(|| {
                    for dir_entry in read_dir {
                        match dir_entry {
                            Ok(entry) => {
                                let is_dir = entry.path().is_dir();

                                let f = entry.file_name();
                                let filename = f.to_string_lossy();
                                let modification_date = entry
                                    .metadata()
                                    .ok()
                                    .and_then(|x| x.modified().ok())
                                    .and_then(|x| x.duration_since(UNIX_EPOCH).ok())
                                    .map(|x| DateTime::with_millis(x.as_millis()));

                                let byte_size = entry.metadata().ok().map(|x| x.len());

                                html::tr(|| {
                                    html::td(|| {
                                        html::a(|| {
                                            let link = format!("{dir_name}/{filename}");
                                            html::attr("href", link.clone());
                                            let icon = if is_dir { "📂" } else { "📄" };
                                            html::content(format!("{icon} {link}"));
                                        });
                                    });

                                    html::td(
                                        modification_date
                                            .map(|x| x.to_iso_8601_string())
                                            .unwrap_or_default(),
                                    );

                                    html::td(
                                        byte_size
                                            .map(|size| format!("{size} bytes"))
                                            .unwrap_or_default(),
                                    );
                                });
                            }
                            Err(err) => {
                                eprintln!("Failed to read: {err}")
                            }
                        }
                    }
                });
            });
        });
    }))
}
