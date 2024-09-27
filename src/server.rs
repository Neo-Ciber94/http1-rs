use std::net::{SocketAddr, TcpListener};

use crate::{
    handler::RequestHandler,
    runtime::{runtime::Runtime, DefaultRuntime},
};

/// Server configuration.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whether if include the `Date` header in each request.
    pub include_date_header: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include_date_header: true,
        }
    }
}

type OnReady = Option<Box<dyn FnOnce(&SocketAddr)>>;

/// The server implementation.
pub struct Server {
    addr: SocketAddr,
    config: Config,
    on_ready: OnReady,
}

impl Server {
    /// Constructs a new server.
    pub fn new(addr: SocketAddr) -> Self {
        let config = Config::default();

        Server {
            addr,
            config,
            on_ready: None,
        }
    }

    /// Adds a callback that will be executed right after the server starts.
    pub fn on_ready<F: FnOnce(&SocketAddr) + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Whether if include the `Date` header.
    pub fn include_date_header(mut self, include: bool) -> Self {
        self.config.include_date_header = include;
        self
    }

    /// Starts listening for requests using the default http1 `Runtime`.
    pub fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        handler: H,
    ) -> std::io::Result<()> {
        self.start_with(DefaultRuntime::default(), handler)
    }

    /// Starts listening for requests using the given `Runtime`.
    pub fn start_with<E: Runtime, H: RequestHandler + Send + Sync + 'static>(
        self,
        engine: E,
        handler: H,
    ) -> std::io::Result<E::Output> {
        let Self {
            addr,
            config,
            mut on_ready,
        } = self;

        let listener = TcpListener::bind(addr)?;

        if let Some(on_ready) = on_ready.take() {
            on_ready(&addr)
        }

        engine.start(listener, config, handler)
    }
}
