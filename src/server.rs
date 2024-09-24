use std::net::SocketAddr;

use crate::{
    handler::RequestHandler,
    http::engine::{default_engine::DefaultEngine, Engine, EngineStartInfo},
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
        self.start_with(DefaultEngine, handler)
    }

    pub fn start_with<E: Engine, H: RequestHandler + Send + Sync + 'static>(
        self,
        engine: E,
        handler: H,
    ) -> E::Ret {
        let Self {
            addr,
            config,
            on_ready,
        } = self;

        engine.start(EngineStartInfo {
            addr,
            config,
            handler,
            on_ready,
        })
    }
}
