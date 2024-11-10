use std::sync::{Arc, Mutex};

use crate::{common::thread_pool::ThreadPool, protocol::h1::handle_incoming};

use super::runtime::{Runtime, StartRuntime};

pub struct ThreadPooledRuntime(ThreadPool);

impl ThreadPooledRuntime {
    pub fn new() -> std::io::Result<Self> {
        let parallelism = std::thread::available_parallelism()?;
        let pool = ThreadPool::builder()
            .num_workers(parallelism.get())
            .on_worker_panic(|| {
                log::error!(
                    "Worker thread panicked: {:?}",
                    std::thread::current().name()
                );
            })
            .build()?;

        Ok(Self::with_pool(pool))
    }

    pub fn with_pool(pool: ThreadPool) -> Self {
        ThreadPooledRuntime(pool)
    }

    pub fn pool(&self) -> &ThreadPool {
        &self.0
    }
}

impl Runtime for ThreadPooledRuntime {
    type Output = ();

    fn start<H: crate::handler::RequestHandler + Send + Sync + 'static>(
        self,
        args: StartRuntime,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        let StartRuntime {
            listener,
            config,
            handle,
        } = args;

        let signal = handle.shutdown_signal;
        let pool = &self.0;
        let handler = Mutex::new(Arc::new(handler));

        loop {
            if signal.is_stopped() {
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => {
                    let lock = handler.lock().expect("Failed to acquire handler lock");
                    let config = config.clone();
                    let handler = lock.clone();

                    pool.execute(move || match handle_incoming(&handler, &config, stream) {
                        Ok(_) => {}
                        Err(err) => log::error!("{err}"),
                    })?;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
