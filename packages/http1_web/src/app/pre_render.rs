use std::{
    collections::HashSet,
    fs::OpenOptions,
    path::Path,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use http1::{
    body::{body_reader::BodyReader, http_body::HttpBody},
    client::Client,
    common::uuid::Uuid,
    headers::HeaderName,
    server::Server,
};

use super::App;

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

pub fn pre_render(
    app: Arc<App>,
    destination_dir: impl AsRef<Path>,
    config: PreRenderConfig,
) -> std::io::Result<()> {
    let path = destination_dir.as_ref();

    if path.exists() && !path.is_dir() {
        return Err(std::io::Error::other(format!(
            "{path:?} is not a directory"
        )));
    }

    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    let included_routes = match config.include {
        Some(r) => r,
        None => app
            .routes()
            .filter_map(|r| {
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

    log::info!("Starting temporal server for prerendering");
    std::thread::spawn(move || server.listen(addr, app).expect("server failed"));

    // Pre-render the routes
    log::info!("Prerendering routes: {included_routes:?}");

    let exclude = config.exclude.unwrap_or_default();
    let pre_render_count = Arc::new(AtomicUsize::new(0));

    let pre_render_id = Uuid::new_v4().to_string();
    let client = Arc::new(
        Client::builder()
            .insert_default_header(
                HeaderName::from_static("Route-Pre-Render"),
                pre_render_id.clone(),
            )
            .read_timeout(Some(Duration::from_millis(5000)))
            .build(),
    );

    let routes = included_routes
        .difference(&exclude)
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();

    std::thread::scope(|s| {
        for route in routes {
            let client = Arc::clone(&client);
            let pre_render_count = Arc::clone(&pre_render_count);

            s.spawn(move || {
                let base_url = format!("http://127.0.0.1:{port}");
                let url = format!("{base_url}/{route}");
                let dst_dir = path.to_path_buf();

                match client.get(url.as_str()).send(()) {
                    Ok(res) => {
                        if !res.status().is_success() {
                            log::error!(
                                "Failed to pre-render `{url}`, returned {} status code",
                                res.status()
                            );

                            return;
                        }

                        let dst_file = dst_dir.join(route);
                        let mut dst = OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(&dst_file)
                            .expect("failed to create file");

                        let body = res.into_body();
                        let mut src = BodyReader::new(body);
                        std::io::copy(&mut src, &mut dst).unwrap_or_else(|_| {
                            panic!("failed to write response contents from `{url}`")
                        });

                        pre_render_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        log::debug!("Written pre-render contents from `{url}` to `{dst_file:?}`");
                    }
                    Err(err) => log::error!("Failed to pre-render `{url}`: {err}"),
                }
            });
        }
    });

    log::info!(
        "{} routes were pre-rendered, written to {path:?}",
        pre_render_count.load(std::sync::atomic::Ordering::Relaxed)
    );

    server_handle.shutdown();
    Ok(())
}