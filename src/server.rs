use std::net::SocketAddr;

use crate::{
    handler::RequestHandler,
    http::engine::{simple_engine::SimpleEngine, Engine, EngineStartInfo},
};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub include_date_header: bool,
}

pub struct Server<E> {
    addr: SocketAddr,
    on_ready: Option<Box<dyn FnOnce(&SocketAddr)>>,
    config: ServerConfig,
    engine: E,
}

impl Server<()> {
    pub fn new(addr: SocketAddr) -> Server<SimpleEngine> {
        Server::with_engine(addr, SimpleEngine)
    }

    pub fn with_engine<E>(addr: SocketAddr, engine: E) -> Server<E> {
        Server {
            addr,
            engine,
            on_ready: None,
            config: ServerConfig {
                include_date_header: true,
            },
        }
    }
}

impl<E> Server<E>
where
    E: Engine,
{
    pub fn on_ready<F: FnOnce(&SocketAddr) + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    pub fn include_date_header(mut self, include: bool) -> Self {
        self.config.include_date_header = include;
        self
    }

    pub fn start<H: RequestHandler + Send + Sync + 'static>(self, handler: H) -> E::Ret {
        let Self {
            addr,
            config,
            on_ready,
            engine,
        } = self;

        engine.start(EngineStartInfo {
            addr,
            config,
            handler,
            on_ready,
        })
    }
}
