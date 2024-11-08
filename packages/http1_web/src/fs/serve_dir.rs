use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    from_request::FromRequestRef,
    handler::Handler,
    html::{self, element::HTMLElement},
    mime::Mime,
    routing::route_info::RouteInfo,
    ErrorResponse, ErrorStatusCode, IntoResponse,
};
use datetime::DateTime;
use http1::{
    body::Body, headers, method::Method, request::Request, response::Response, status::StatusCode,
};

pub struct DefaultServeDirFallback;

impl Handler<Request<Body>> for DefaultServeDirFallback {
    type Output = Response<Body>;

    fn call(&self, _: Request<Body>) -> Self::Output {
        Response::new(StatusCode::NOT_FOUND, Body::empty())
    }
}

/// A handler to serve static files from a path.
#[derive(Debug)]
pub struct ServeDir<F = DefaultServeDirFallback> {
    root: PathBuf,
    index_html: bool,
    list_directory: bool,
    fallback: F,
}

impl ServeDir<()> {
    pub fn new(dir: impl AsRef<Path>) -> ServeDir<DefaultServeDirFallback> {
        Self::with_fallback(dir, DefaultServeDirFallback)
    }
}

impl<F> ServeDir<F> {
    /// Constructs a new `ServeDir`
    ///
    /// # Parameters
    /// - `dir` to directory in the file system relative to the `cwd` to serve the files from.
    /// - `fallback` the fallback service.
    pub fn with_fallback<A: Handler<Request<Body>, Output = Response<Body>>>(
        dir: impl AsRef<Path>,
        fallback: A,
    ) -> ServeDir<A> {
        let cwd = std::env::current_dir().expect("unable to get current directory");
        let path = dir.as_ref();
        let root = cwd.join(path);

        assert!(root.is_dir(), "`{root:?}` is not a directory");

        ServeDir {
            root,
            index_html: false,
            list_directory: false,
            fallback,
        }
    }

    /// Whether if try to append `/index.html` when request a directory, defaults to false.
    ///
    /// # Example
    /// - `/hello` will try to match `/hello.html` and `/hello/index.html` and send those html files any exists.
    pub fn append_html_index(mut self, index_html: bool) -> Self {
        self.index_html = index_html;
        self
    }

    /// Enables directory listing. By default is `false`.
    pub fn list_directory(mut self, list_directory: bool) -> Self {
        self.list_directory = list_directory;
        self
    }
}

impl<F> Handler<Request<Body>> for ServeDir<F>
where
    F: Handler<Request<Body>, Output = Response<Body>>,
{
    type Output = Response<Body>;

    fn call(&self, req: Request<Body>) -> Self::Output {
        if !(req.method() == Method::GET || req.method() == Method::HEAD) {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }

        let route_info = RouteInfo::from_request_ref(&req).unwrap();
        let route = get_route(route_info, req.uri().path_and_query().path());
        let mut serve_path = self.root.join(&route);

        if serve_path.is_dir() && self.index_html {
            serve_path = serve_path.join("index.html")
        }

        log::debug!("serving path: {serve_path:?}");

        if !serve_path.exists() {
            return self.fallback.call(req);
        }

        let mime = serve_path
            .extension()
            .and_then(|x| x.to_str())
            .and_then(|ext| Mime::from_extension(ext).ok())
            .unwrap_or(Mime::APPLICATION_OCTET_STREAM);

        if serve_path.is_dir() {
            if self.list_directory {
                return list_directory_html(&route, &serve_path).into_response();
            } else {
                return StatusCode::NOT_FOUND.into_response();
            }
        }

        match File::open(&serve_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Response::builder()
                    .append_header(headers::CONTENT_TYPE, mime.to_string())
                    .body(Body::new(reader))
            }
            Err(err) => {
                log::error!("Failed to open file: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

fn get_route(route_info: RouteInfo, req_path: &str) -> String {
    let segments_count = route_info.iter().filter(|x| !x.is_catch_all()).count();

    let mut path = String::new();

    for (idx, segment) in crate::routing::route::get_segments(req_path).enumerate() {
        if idx < segments_count {
            continue;
        }

        if !path.is_empty() {
            path.push_str("/");
        }

        path.push_str(segment);
    }

    path
}

fn list_directory_html(route: &str, dir: &Path) -> Result<HTMLElement, ErrorResponse> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| {
        log::error!("Failed to list directory: {err}");
        ErrorResponse::new(ErrorStatusCode::InternalServerError, ())
    })?;

    let dir_name = if route.starts_with("/") {
        route.to_string()
    } else {
        format!("/{route}")
    };

    Ok(html::html(|| {
        html::head(|| {
            html::title(dir_name.clone());

            html::meta(|| {
                html::attr("charset", "UTF-8");
                html::attr("content", "width=device-width, initial-scale=1.0");
            });

            html::style(
                r#"
            body {
                font-family: Arial, sans-serif;
                margin: 20px;
            }
            table {
                width: 100%;
                border-collapse: collapse;
            }
            th, td {
                border: 1px solid #ddd;
                padding: 8px;
                text-align: left;
            }
            th {
                background-color: #f2f2f2;
            }
            "#,
            );
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
                    if !route.is_empty() {
                        html::tr(|| {
                            html::td(|| {
                                html::a(|| {
                                    html::attr("href", format!("{dir_name}/.."));
                                    html::content("../");
                                });
                            });

                            html::td("-");
                            html::td("-");
                        });
                    }

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
                                            .unwrap_or_else(|| String::from("-")),
                                    );

                                    html::td(
                                        byte_size
                                            .map(|size| format!("{size} bytes"))
                                            .unwrap_or_else(|| String::from("-")),
                                    );
                                });
                            }
                            Err(err) => {
                                log::error!("Failed to read: {err}")
                            }
                        }
                    }
                });
            });
        });
    }))
}
