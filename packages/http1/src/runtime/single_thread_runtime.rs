use crate::protocol::h1::handle_incoming;

use super::runtime::{Runtime, StartRuntime};

pub struct SingleThreadRuntime;

impl Runtime for SingleThreadRuntime {
    type Output = ();

    fn start<H: crate::handler::RequestHandler + Send + Sync + 'static>(
        self,
        args: StartRuntime,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        let StartRuntime {
            config,
            handle,
            listener,
        } = args;

        let signal = handle.shutdown_signal;

        loop {
            if signal.is_stopped() {
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => match handle_incoming(&handler, &config, stream) {
                    Ok(_) => {}
                    Err(err) => log::error!("{err}"),
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}
