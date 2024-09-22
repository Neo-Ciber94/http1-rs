pub mod threadpooled_engine;

use std::net::SocketAddr;
use crate::{handler::RequestHandler, server::ServerConfig};

pub struct ServerStartInfo<H: RequestHandler> {
    pub addr: SocketAddr,
    pub on_ready: Option<Box<dyn FnOnce(&SocketAddr)>>,
    pub config: ServerConfig,
    pub handler: H,
}

pub trait Engine {
    type Ret;

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        info: ServerStartInfo<H>,
    ) -> Self::Ret;
}
