use std::{
    fmt::Debug,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

use crate::{
    from_request::FromRequest,
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

type OnResponseHandler = Arc<Mutex<dyn FnMut(&mut Response<Body>) + Send>>;

/// A handler to serve static files from a path.
pub struct ServeDir<F = DefaultServeDirFallback, R = ResolveIndexHtml> {
    root: PathBuf,
    list_directory: bool,
    use_cache_headers: bool,
    fallback: F,
    on_response: Option<OnResponseHandler>,
    resolve_index: Option<R>,
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
    ) -> ServeDir<A, ResolveIndexHtml> {
        let cwd = std::env::current_dir().expect("unable to get current directory");
        let path = dir.as_ref();
        let root = cwd.join(path);

        assert!(root.is_dir(), "`{root:?}` is not a directory");

        ServeDir {
            root,
            list_directory: false,
            use_cache_headers: false,
            on_response: None,
            fallback,
            resolve_index: None,
        }
    }

    /// Whether if resolve requests to `/<path>` to `/<path>.html` when required.
    pub fn resolve_index_html(self) -> Self {
        self.resolve_index_with(ResolveIndexHtml)
    }

    /// Whether if resolve requests to `/<path>` to `/<path>.<extension>` when required.
    pub fn resolve_index_to_any(self) -> ServeDir<F, ResolveAnyIndex> {
        self.resolve_index_with(ResolveAnyIndex)
    }

    /// Resolve index file using the given `ResolveIndex`.
    pub fn resolve_index_with<U>(self, resolve_index: U) -> ServeDir<F, U>
    where
        U: ResolveIndex,
    {
        ServeDir {
            root: self.root,
            fallback: self.fallback,
            list_directory: self.list_directory,
            on_response: self.on_response,
            resolve_index: Some(resolve_index),
            use_cache_headers: self.use_cache_headers,
        }
    }

    /// Enables directory listing. By default is `false`.
    pub fn list_directory(mut self, list_directory: bool) -> Self {
        self.list_directory = list_directory;
        self
    }

    /// Whether if append `Cache-Control` headers for the serve files.
    pub fn use_cache_headers(mut self, use_cache_headers: bool) -> Self {
        self.use_cache_headers = use_cache_headers;
        self
    }

    /// Add a handler that run each time a response is about to be send.
    pub fn on_response<U>(mut self, on_response: U) -> Self
    where
        U: FnMut(&mut Response<Body>) + Send + 'static,
    {
        self.on_response = Some(Arc::new(Mutex::new(on_response)));
        self
    }
}

impl Debug for ServeDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServeDir")
            .field("root", &self.root)
            .field("list_directory", &self.list_directory)
            .field("use_cache_headers", &self.use_cache_headers)
            .finish_non_exhaustive()
    }
}

impl<F, R> Handler<Request<Body>> for ServeDir<F, R>
where
    F: Handler<Request<Body>, Output = Response<Body>>,
    R: ResolveIndex,
{
    type Output = Response<Body>;

    fn call(&self, req: Request<Body>) -> Self::Output {
        if !(req.method() == Method::GET || req.method() == Method::HEAD) {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }

        let (req, mut extensions, mut payload) = crate::from_request::split_request(req);
        let route_info = RouteInfo::from_request(&req, &mut extensions, &mut payload).unwrap();
        let req_path = req.uri().path_and_query().path();
        let route = get_route(route_info, req_path);
        let mut serve_path = self.root.join(&route);

        if let Some(index) = &self.resolve_index {
            serve_path = match index.resolve_index(&req, serve_path) {
                Ok(p) => p,
                Err(err) => return err.into_response(),
            }
        }

        let mime = serve_path
            .extension()
            .and_then(|x| x.to_str())
            .and_then(|ext| Mime::from_extension(ext).ok())
            .unwrap_or(Mime::APPLICATION_OCTET_STREAM);

        if serve_path.is_dir() {
            if self.list_directory {
                return list_directory_html(req_path, &serve_path).into_response();
            } else {
                let req = crate::from_request::join_request(req, extensions, payload);
                return self.fallback.call(req);
            }
        }

        if !serve_path.exists() {
            let req = crate::from_request::join_request(req, extensions, payload);
            return self.fallback.call(req);
        }

        log::debug!("serving path: {serve_path:?}");

        match File::open(&serve_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut res = Response::builder()
                    .append_header(headers::CONTENT_TYPE, mime.to_string())
                    .body(Body::new(reader));

                if self.use_cache_headers {
                    res.headers_mut().insert(
                        headers::CACHE_CONTROL,
                        String::from("public, max-age=604800, immutable"),
                    );
                }

                if let Some(mtx) = self.on_response.as_ref() {
                    let mut on_response = mtx.lock().expect("failed to get on_response");
                    (on_response)(&mut res);
                }

                res
            }
            Err(err) => {
                log::error!("Failed to open file: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

fn list_directory_html(req_path: &str, dir: &Path) -> Result<HTMLElement, ErrorResponse> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| {
        log::error!("Failed to list directory: {err}");
        ErrorResponse::new(ErrorStatusCode::InternalServerError, ())
    })?;

    let req_segments = crate::routing::route::get_segments(req_path);
    let route = interspace("/", req_segments).collect::<String>();
    let dir_name = format!("/{route}");

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

fn get_route(route_info: RouteInfo, req_path: &str) -> String {
    let segments_count = route_info.iter().filter(|x| !x.is_catch_all()).count();
    let segments = crate::routing::route::get_segments(req_path);
    interspace("/", segments.skip(segments_count)).collect::<String>()
}

fn interspace<T, I>(separator: T, iter: I) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    T: Clone,
{
    let mut iter = iter.peekable();
    let mut insert_separator = false;

    std::iter::from_fn(move || {
        if insert_separator && iter.peek().is_some() {
            insert_separator = false;
            Some(separator.clone())
        } else if let Some(next) = iter.next() {
            insert_separator = true;
            Some(next)
        } else {
            None
        }
    })
}

/**
 * Resolve the location of a file.
 */
pub trait ResolveIndex {
    /// Returns the path for a file.
    fn resolve_index(&self, req: &Request<()>, serve_path: PathBuf) -> std::io::Result<PathBuf>;
}

/// Resolves request to `/<path>` to `/index.html` or `/<path>.html` when required.
pub struct ResolveIndexHtml;
impl ResolveIndex for ResolveIndexHtml {
    fn resolve_index(&self, _req: &Request<()>, serve_path: PathBuf) -> std::io::Result<PathBuf> {
        // Try to match /index.html
        if serve_path.is_dir() {
            let index_html = serve_path.join("index.html");

            if index_html.exists() {
                return Ok(index_html);
            }
        }

        // Try to match /<path>.html
        if !serve_path.exists() {
            let mut html_file = serve_path.clone();
            html_file.set_extension("html");

            if html_file.exists() {
                return Ok(html_file);
            }
        }

        Ok(serve_path)
    }
}

/// Attempt to resolve request to `/<path>` to a matching file in the file system.
pub struct ResolveAnyIndex;
impl ResolveAnyIndex {
    fn resolve_any(serve_path: PathBuf) -> std::io::Result<PathBuf> {
        // We use this ones for a fast look up
        static COMMON_MIMES: &[Mime] = &[
            Mime::TEXT_HTML,
            Mime::TEXT_PLAIN,
            Mime::APPLICATION_JSON,
            Mime::APPLICATION_XML,
            Mime::APPLICATION_PDF,
        ];

        for mime in COMMON_MIMES {
            if let Some(p) = Self::try_index_type(&serve_path, mime) {
                return Ok(p);
            }
        }

        // Try to find the first file that matches `/index.{any-extension}`
        let target_dir = if serve_path.is_dir() {
            Some(serve_path.as_path())
        } else {
            serve_path.ancestors().skip(1).next()
        };

        if target_dir.is_none() {
            return Ok(serve_path);
        }

        if let Some(path) = target_dir {
            let read_dir = std::fs::read_dir(path)?;

            for entry in read_dir {
                if let Ok(entry) = entry {
                    let file_path = entry.path();

                    if serve_path.is_dir() {
                        let is_index_file = file_path
                            .file_stem()
                            .is_some_and(|s| s.to_string_lossy() == "index");

                        if is_index_file && file_path.is_file() && file_path.extension().is_some() {
                            return Ok(file_path);
                        }
                    } else if let Some(f) = serve_path.file_name() {
                        let is_match = file_path.file_stem().is_some_and(|s| s == f);
                        if is_match && file_path.is_file() && file_path.extension().is_some() {
                            return Ok(file_path);
                        }
                    }
                }
            }
        }

        Ok(serve_path)
    }

    fn resolve_from_types(serve_path: PathBuf, types: &[Mime]) -> std::io::Result<PathBuf> {
        for mime in types {
            if let Some(p) = Self::try_index_type(&serve_path, mime) {
                return Ok(p);
            }
        }

        Ok(serve_path)
    }

    fn try_index_type(serve_path: &Path, mime: &Mime) -> Option<PathBuf> {
        let ext = mime.extension()?;

        // Try to match /index.{ext}
        if serve_path.is_dir() {
            let index_file = serve_path.join(format!("index.{ext}"));
            dbg!(&index_file);

            if index_file.exists() {
                return Some(index_file);
            }
        }

        // Try to match /<path>.{ext}
        if !serve_path.exists() {
            let mut index_file = serve_path.to_path_buf().clone();
            index_file.set_extension(ext);

            dbg!(&index_file);
            if index_file.exists() {
                return Some(index_file);
            }
        }

        None
    }
}

impl ResolveIndex for ResolveAnyIndex {
    fn resolve_index(&self, req: &Request<()>, serve_path: PathBuf) -> std::io::Result<PathBuf> {
        let mut accept_header = req.headers().get_all(headers::ACCEPT).peekable();

        if accept_header.peek().is_none() {
            return ResolveAnyIndex::resolve_any(serve_path);
        }

        let types = accept_header
            .filter_map(|x| Mime::from_str(x.as_str()).ok())
            .collect::<Vec<_>>();

        if types.iter().any(|x| x.ty() == "*" && x.subtype() == "*") {
            return ResolveAnyIndex::resolve_any(serve_path);
        }

        ResolveAnyIndex::resolve_from_types(serve_path, &types)
    }
}
