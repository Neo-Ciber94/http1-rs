use std::{
    collections::HashSet,
    convert::Infallible,
    fs::OpenOptions,
    path::Path,
    str::FromStr,
    sync::{atomic::AtomicUsize, Arc, LazyLock},
    thread::ScopedJoinHandle,
    time::Duration,
};

use http1::{
    body::{body_reader::BodyReader, Body},
    client::Client,
    common::uuid::Uuid,
    error::BoxError,
    headers::{self, HeaderName, HeaderValue},
    method::Method,
    response::Response,
    server::Server,
};

use crate::{from_request::FromRequest, middleware::extensions::ExtensionsProvider, mime::Mime};

use super::App;

static HEADER_PRE_RENDER: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("Pre-Render"));

#[derive(Debug, Default)]
pub struct PreRenderConfig {
    port: Option<u16>,
    include: Option<HashSet<String>>,
    exclude: Option<HashSet<String>>,
}

impl PreRenderConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn port(&mut self, port: u16) {
        self.port = Some(port);
    }

    pub fn include(&mut self, route: impl Into<String>) {
        self.include
            .get_or_insert_with(Default::default)
            .insert(route.into());
    }

    pub fn exclude(&mut self, route: impl Into<String>) {
        self.include
            .get_or_insert_with(Default::default)
            .insert(route.into());
    }
}

#[derive(Debug, Clone)]
struct PreRenderId(String);

/// Allow to check if the current route is being pre-rended.
#[derive(Debug, Clone)]
pub struct PreRendering(bool);
impl PreRendering {
    pub fn is_pre_rendering(&self) -> bool {
        self.0
    }
}

impl FromRequest for PreRendering {
    type Rejection = Infallible;

    fn from_request(
        req: &http1::request::Request<()>,
        _payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        match req.extensions().get::<PreRenderId>().cloned() {
            Some(PreRenderId(id)) => {
                let is_pre_rendering = req
                    .headers()
                    .get(HEADER_PRE_RENDER.as_str())
                    .is_some_and(|x| x.as_str() == id.as_str());
                Ok(PreRendering(is_pre_rendering))
            }
            None => Ok(PreRendering(false)),
        }
    }
}

/// Pre-render the routes of `App`.
pub fn pre_render(
    mut app: App,
    destination_dir: impl AsRef<Path>,
    config: PreRenderConfig,
) -> std::io::Result<()> {
    let cwd = std::env::current_dir()?;
    let target_dir = cwd.join(destination_dir.as_ref());

    if target_dir.exists() && !target_dir.is_dir() {
        return Err(std::io::Error::other(format!(
            "{target_dir:?} is not a directory"
        )));
    }

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
    }

    let included_routes = match config.include {
        Some(r) => r,
        None => app
            .routes()
            .filter_map(|(r, method)| {
                if method != Method::GET {
                    return None;
                }
                
                if r.is_static() {
                    return Some(r.to_string());
                }

                log::warn!("Skipping dynamic route `{r}`, use `PreRenderConfig::include` to include dynamic routes to pre-render");
                None
            })
            .collect(),
    };

    // Start the server
    let port = config
        .port
        .unwrap_or_else(|| http1::common::find_open_port::find_open_port().unwrap());

    let addr = ("127.0.0.1", port);
    let server = Server::new();
    let server_handle = server.handle();

    log::info!("Starting server for prerendering");
    let pre_render_id = Uuid::new_v4().to_string();
    app.add_middleware(ExtensionsProvider::new().insert(PreRenderId(pre_render_id.clone())));
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();

    std::thread::spawn(move || {
        server
            .on_ready(move |_| {
                ready_tx.send(()).unwrap();
                log::info!("Pre-rendering server had started");
            })
            .listen(addr, app)
            .expect("server failed")
    });

    // Pre-render the routes
    log::info!("Prerendering routes: {included_routes:?}");

    let pre_render_count = Arc::new(AtomicUsize::new(0));
    let exclude: HashSet<String> = config.exclude.unwrap_or_default();
    let routes = included_routes.difference(&exclude);

    // Wait for the server to start
    std::thread::sleep(Duration::from_millis(100));
    ready_rx
        .recv_timeout(Duration::from_millis(1000))
        .expect("server startup timeout");

    pre_render_routes(
        routes,
        pre_render_id,
        Arc::clone(&pre_render_count),
        port,
        &target_dir,
    )
    .expect("failed to pre-render");

    log::info!(
        "{} routes were pre-rendered, written to {target_dir:?}",
        pre_render_count.load(std::sync::atomic::Ordering::Relaxed)
    );

    server_handle.shutdown();
    Ok(())
}

fn pre_render_routes<'a>(
    routes: impl IntoIterator<Item = &'a String>,
    pre_render_id: String,
    pre_render_count: Arc<AtomicUsize>,
    port: u16,
    target_dir: &Path,
) -> Result<(), BoxError> {
    let client = Arc::new(
        Client::builder()
            .insert_default_header(
                (*HEADER_PRE_RENDER).clone(),
                HeaderValue::try_from(pre_render_id.clone())?,
            )
            .read_timeout(Some(Duration::from_millis(5000)))
            .build(),
    );

    let errors = std::thread::scope(|s| {
        let mut handles = Vec::new();

        for route in routes {
            let client = Arc::clone(&client);
            let pre_render_count = Arc::clone(&pre_render_count);

            let handle: ScopedJoinHandle<Result<(), BoxError>> = s.spawn(move || {
                let base_url = format!("http://127.0.0.1:{port}");
                let url = format!("{base_url}{route}");

                let response = client
                    .get(url.as_str())
                    .send(())
                    .map_err(|err| format!("Failed to pre-render `{url}`: {err}"))?;

                if !response.status().is_success() {
                    return Err(format!(
                        "Failed to pre-render `{url}`, returned {} status code",
                        response.status()
                    )
                    .into());
                }

                write_response_to_fs(response, &url, target_dir, route);
                pre_render_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            });

            handles.push(handle);
        }

        handles
            .into_iter()
            .map(|h| h.join())
            .filter_map(|x| x.ok())
            .filter(|x| x.is_err())
            .map(|x| x.unwrap_err())
            .collect::<Vec<_>>()
    });

    if !errors.is_empty() {
        let error_msg = errors
            .into_iter()
            .map(|err| format!("\t{err}"))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(format!("pre-rendering error:\n\n{error_msg}").into());
    }

    Ok(())
}

fn write_response_to_fs(response: Response<Body>, url: &str, target_dir: &Path, mut route: &str) {
    if route == "/" {
        route = "/index"
    }

    let mut dst_file = target_dir.join(route.strip_prefix("/").unwrap_or(route));

    if let Some(parent) = dst_file.ancestors().nth(1) {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("failed to create parent dir for: `{route}`"));
        }
    }

    let has_extensions = dst_file.extension().is_some();

    if !has_extensions {
        if let Some(mime) = response
            .headers()
            .get(headers::CONTENT_TYPE)
            .and_then(|s| Mime::from_str(s.as_str()).ok())
        {
            if let Some(ext) = mime.extension() {
                dst_file.set_extension(ext);
            }
        }
    }

    let mut dst = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&dst_file)
        .expect("failed to create file");

    let body = response.into_body();
    let mut src = BodyReader::new(body);

    std::io::copy(&mut src, &mut dst)
        .unwrap_or_else(|_| panic!("failed to write response contents from `{url}`"));

    log::debug!("Written pre-render contents from `{url}` to `{dst_file:?}`");
}
