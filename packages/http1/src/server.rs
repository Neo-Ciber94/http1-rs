use std::{
    net::{SocketAddr, TcpListener, ToSocketAddrs},
    sync::{atomic::AtomicBool, Arc},
};

use crate::{
    handler::RequestHandler,
    runtime::{
        runtime::{Runtime, StartRuntime},
        DefaultRuntime,
    },
};

#[derive(Clone)]
pub struct ShutdownSignal(Arc<AtomicBool>);

impl ShutdownSignal {
    fn new() -> Self {
        ShutdownSignal(Arc::default())
    }

    /// Signal the server to stop.
    pub fn shutdown(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }

    /// Whether the server was signal to shutdown.
    pub fn is_stopped(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct ServerHandle {
    pub signal: ShutdownSignal,
}

/// Server configuration.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whether if include the `Date` header in each request.
    pub include_date_header: bool,

    /// Max body size in bytes.
    pub max_body_size: Option<usize>,

    /// Whether if include connection information to the request extensions.
    pub include_conn_info: bool,

    /// Whether if include the server config to the request extensions.
    pub include_server_info: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include_date_header: true,
            max_body_size: Some(crate::constants::DEFAULT_MAX_BODY_SIZE),
            include_conn_info: false,
            include_server_info: true,
        }
    }
}

type OnReady = Option<Box<dyn FnOnce(&SocketAddr) + Send>>;

/// The server implementation.
pub struct Server<A> {
    addr: A,
    config: Config,
    on_ready: OnReady,
    handle: ServerHandle,
}

impl Server<()> {
    /// Constructs a new server.
    pub fn new<A: ToSocketAddrs>(addr: A) -> Server<A> {
        let config = Config::default();
        let handle = ServerHandle {
            signal: ShutdownSignal::new(),
        };

        Server {
            addr,
            config,
            handle,
            on_ready: None,
        }
    }
}

impl<A> Server<A> {
    /// Whether if include the `Date` header.
    pub fn include_date_header(mut self, include: bool) -> Self {
        self.config.include_date_header = include;
        self
    }

    /// The max size in bytes the body is allowed to have, if the body surpasses that size the request will be rejected.
    pub fn max_body_size(mut self, max_body_size_bytes: Option<usize>) -> Self {
        if let Some(max_body_size_bytes) = max_body_size_bytes {
            assert!(max_body_size_bytes > 0);
        }

        self.config.max_body_size = max_body_size_bytes;
        self
    }

    /// Include connection information to the request extensions.
    pub fn include_conn_info(mut self, insert_conn_info: bool) -> Self {
        self.config.include_conn_info = insert_conn_info;
        self
    }

    /// Include the server config to the request extensions.
    pub fn include_server_info(mut self, include_server_info: bool) -> Self {
        self.config.include_server_info = include_server_info;
        self
    }

    /// Returns a handle to stop the server.
    pub fn handle(&self) -> ServerHandle {
        self.handle.clone()
    }
}

impl<A: ToSocketAddrs> Server<A> {
    /// Adds a callback that will be executed right after the server starts.
    pub fn on_ready<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&SocketAddr) + Send + 'static,
    {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Starts listening for requests using the default http1 `Runtime`.
    pub fn start<H>(self, handler: H) -> std::io::Result<()>
    where
        H: RequestHandler + Send + Sync + 'static,
    {
        self.start_with(DefaultRuntime::default(), handler)
    }

    /// Starts listening for requests using the given `Runtime`.
    pub fn start_with<R, H>(self, runtime: R, handler: H) -> std::io::Result<R::Output>
    where
        R: Runtime,
        H: RequestHandler + Send + Sync + 'static,
    {
        let Self {
            addr,
            config,
            handle,
            mut on_ready,
        } = self;

        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?;

        if let Some(on_ready) = on_ready.take() {
            on_ready(&local_addr)
        }

        let args = StartRuntime {
            listener,
            config,
            handle,
        };

        runtime.start(args, handler)
    }
}
