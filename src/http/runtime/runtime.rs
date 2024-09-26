use crate::{handler::RequestHandler, server::Config};
use std::net::TcpListener;

/// A runtime to handle requests.
pub trait Runtime {
    /// The result when running the runtime.
    type Output;

    /// Starts running the server an accepting requests.
    /// 
    /// To correctly implement a `Runtime` you forward the stream receives from the listener to `http1::protocol::h1::handle_incoming` 
    /// which manages http1 connections.
    /// 
    /// # Parameters
    /// - `listener`: The socket that receives the connections.
    /// - `config`: The additional server configuration.
    /// - `handler`: The request handler.
    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: Config,
        handler: H,
    ) -> std::io::Result<Self::Output>;
}
