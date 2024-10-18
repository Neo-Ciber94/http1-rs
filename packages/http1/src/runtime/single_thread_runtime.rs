use crate::protocol::h1::handle_incoming;

use super::runtime::Runtime;

pub struct SingleThreadRuntime;

impl Runtime for SingleThreadRuntime {
    type Output = ();

    fn start<H: crate::handler::RequestHandler + Send + Sync + 'static>(
        self,
        listener: std::net::TcpListener,
        config: crate::server::Config,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = config.clone();
                    match handle_incoming(&handler, &config, stream) {
                        Ok(_) => {}
                        Err(err) => log::error!("{err}"),
                    }
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
