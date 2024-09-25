use std::net::{SocketAddr, TcpListener};

use crate::{
    handler::RequestHandler,
    http::runtime::{runtime::Runtime, DefaultRuntime},
};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub include_date_header: bool,
}

pub struct Server {
    addr: SocketAddr,
    on_ready: Option<Box<dyn FnOnce(&SocketAddr)>>,
    config: ServerConfig,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Server {
        Server {
            addr,
            on_ready: None,
            config: ServerConfig {
                include_date_header: true,
            },
        }
    }

    pub fn on_ready<F: FnOnce(&SocketAddr) + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    pub fn include_date_header(mut self, include: bool) -> Self {
        self.config.include_date_header = include;
        self
    }

    pub fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        handler: H,
    ) -> std::io::Result<()> {
        self.start_with(DefaultRuntime, handler)
    }

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

        let listener = TcpListener::bind(&addr)?;

        if let Some(on_ready) = on_ready.take() {
            on_ready(&addr)
        }

        engine.start(listener, config, handler)
    }
}
