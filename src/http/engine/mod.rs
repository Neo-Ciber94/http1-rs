pub mod default_engine;

use crate::{handler::RequestHandler, server::ServerConfig};
use std::net::SocketAddr;

pub struct EngineStartInfo<H: RequestHandler> {
    pub addr: SocketAddr,
    pub on_ready: Option<Box<dyn FnOnce(&SocketAddr)>>,
    pub config: ServerConfig,
    pub handler: H,
}

pub trait Engine {
    type Ret;

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        info: EngineStartInfo<H>,
    ) -> Self::Ret;
}
