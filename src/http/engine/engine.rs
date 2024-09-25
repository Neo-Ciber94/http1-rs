use crate::{handler::RequestHandler, server::ServerConfig};
use std::net::TcpListener;

pub trait Engine {
    type Output;

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: ServerConfig,
        handler: H,
    ) -> std::io::Result<Self::Output>;
}
