use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

use crate::{handler::RequestHandler, http::protocol::h1::handle_incoming, server::ServerConfig};

use super::runtime::Runtime;

pub struct DefaultRuntime;

impl Runtime for DefaultRuntime {
    type Output = ();

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: ServerConfig,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        let handler = Mutex::new(Arc::new(handler));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = config.clone();
                    let handler_lock = handler.lock().expect("Failed to acquire handler lock");
                    let request_handler = handler_lock.clone();
                    std::thread::spawn(move || handle_incoming(request_handler, config, stream));
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
