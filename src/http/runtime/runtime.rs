use crate::{handler::RequestHandler, server::Config};
use std::net::TcpListener;

/// A runtime to handle requests.
pub trait Runtime {
    /// The result when running the runtime.
    type Output;

    /// Starts running the server an accepting requests.
    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: Config,
        handler: H,
    ) -> std::io::Result<Self::Output>;
}
