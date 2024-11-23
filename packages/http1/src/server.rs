use std::{
    net::{SocketAddr, TcpListener, ToSocketAddrs},
    sync::{atomic::AtomicBool, Arc},
};

use crate::{
    common::thread_pool::ThreadPool,
    handler::RequestHandler,
    protocol::{connection::Connection, h1::handle_incoming},
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
pub struct Server<E = ThreadPool> {
    config: Config,
    on_ready: OnReady,
    handle: ServerHandle,
    executor: E,
}

impl Server<()> {
    /// Constructs a new server.
    pub fn new() -> Server<ThreadPool> {
        let executor = ThreadPool::new().expect("failed to initialize thread pool executor");
        Self::with_executor(executor)
    }

    /// Constructs a new server with the given executor.
    pub fn with_executor<E: Executor>(executor: E) -> Server<E> {
        let config = Config::default();
        let handle = ServerHandle {
            signal: ShutdownSignal::new(),
        };

        Server {
            config,
            handle,
            on_ready: None,
            executor,
        }
    }
}

impl<E> Server<E> {
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

    /// Adds a callback that will be executed right after the server starts.
    pub fn on_ready<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&SocketAddr) + Send + 'static,
    {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Returns a handle to stop the server.
    pub fn handle(&self) -> ServerHandle {
        self.handle.clone()
    }
}

impl<E> Server<E>
where
    E: Executor,
{
    /// Starts listening on the given address and handle incoming request with the given request handler.
    pub fn listen<H, A: ToSocketAddrs>(self, addr: A, handler: H) -> std::io::Result<()>
    where
        H: RequestHandler + Send + Sync + 'static,
    {
        let Self {
            config,
            handle,
            executor,
            mut on_ready,
        } = self;

        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?;

        if let Some(on_ready) = on_ready.take() {
            on_ready(&local_addr)
        }

        let signal = handle.signal;
        let handler = Arc::new(handler);

        loop {
            if signal.is_stopped() {
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => {
                    let config = config.clone();
                    let handler = handler.clone();

                    let result = executor.execute(move || {
                        match handle_incoming(&handler, &config, Connection::Tcp(stream)) {
                            Ok(_) => {}
                            Err(err) => log::error!("{err}"),
                        }
                    });

                    if let Err(err) = result {
                        return Err(err.into());
                    }
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

/// Provides a mechanism for execute tasks.
pub trait Executor {
    type Err: Into<std::io::Error>;

    /// Execute a task.
    fn execute<F>(&self, f: F) -> Result<(), Self::Err>
    where
        F: FnOnce() + Send + 'static;
}

/// An executor that run the task in the same thread.
pub struct BlockingExecutor;
impl Executor for BlockingExecutor {
    type Err = std::io::Error;

    fn execute<F>(&self, f: F) -> Result<(), Self::Err>
    where
        F: FnOnce() + Send + 'static,
    {
        f();
        Ok(())
    }
}

impl Executor for ThreadPool {
    type Err = std::io::Error;

    fn execute<F>(&self, f: F) -> Result<(), Self::Err>
    where
        F: FnOnce() + Send + 'static,
    {
        self.execute(f)
    }
}
