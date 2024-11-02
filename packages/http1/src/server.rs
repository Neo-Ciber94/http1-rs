use std::net::{TcpListener, ToSocketAddrs};

use crate::{
    handler::RequestHandler,
    runtime::{runtime::Runtime, DefaultRuntime},
};

/// Server configuration.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whether if include the `Date` header in each request.
    pub include_date_header: bool,

    /// Max body size in bytes.
    pub max_body_size: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include_date_header: true,
            max_body_size: None,
        }
    }
}

type OnReady<A> = Option<Box<dyn FnOnce(&A)>>;

/// The server implementation.
pub struct Server<A> {
    addr: A,
    config: Config,
    on_ready: OnReady<A>,
}

impl Server<()> {
    /// Constructs a new server.
    pub fn new<A: ToSocketAddrs>(addr: A) -> Server<A> {
        let config = Config::default();

        Server {
            addr,
            config,
            on_ready: None,
        }
    }

    /// Whether if include the `Date` header.
    pub fn include_date_header(mut self, include: bool) -> Self {
        self.config.include_date_header = include;
        self
    }

    /// The max size in bytes the body is allowed to have, if the body surpasses that size the request will be rejected.
    pub fn max_body_size(mut self, max_body_size_bytes: usize) -> Self {
        self.config.max_body_size = Some(max_body_size_bytes);
        self
    }
}

impl<A: ToSocketAddrs> Server<A> {
    /// Adds a callback that will be executed right after the server starts.
    pub fn on_ready<F: FnOnce(&A) + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
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
    pub fn start_with<R: Runtime, H: RequestHandler + Send + Sync + 'static>(
        self,
        runtime: R,
        handler: H,
    ) -> std::io::Result<R::Output> {
        let Self {
            addr,
            config,
            mut on_ready,
        } = self;

        let listener = TcpListener::bind(&addr)?;

        if let Some(on_ready) = on_ready.take() {
            on_ready(&addr)
        }

        runtime.start(listener, config, handler)
    }
}
