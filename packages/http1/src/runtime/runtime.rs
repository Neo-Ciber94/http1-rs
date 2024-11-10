use crate::{
    handler::RequestHandler,
    server::{Config, ServerHandle},
};
use std::net::TcpListener;

/// Configuration to start the runtime.
pub struct StartRuntime {
    /// The socket that receives the connections.
    pub listener: TcpListener,

    /// Server configuration.
    pub config: Config,

    /// The server handle.
    pub handle: ServerHandle,
}

/// The server runtime to handle requests.
pub trait Runtime {
    /// The result when running the runtime.
    type Output;

    /// Starts running the server an accepting requests.
    ///
    /// To correctly implement a `Runtime` you forward the stream receives from the listener to `http1::protocol::h1::handle_incoming`
    /// which manages http1 connections.
    ///
    /// # Parameters
    /// - `args`: The additional configuration.
    /// - `handler`: The request handler.
    fn start<H>(self, args: StartRuntime, handler: H) -> std::io::Result<Self::Output>
    where
        H: RequestHandler + Send + Sync + 'static;
}
