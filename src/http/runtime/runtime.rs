use crate::{handler::RequestHandler, server::Config};
use std::net::TcpListener;

pub trait Runtime {
    type Output;

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: Config,
        handler: H,
    ) -> std::io::Result<Self::Output>;
}
