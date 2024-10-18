use std::sync::{Arc, Mutex};

use crate::{common::thread_pool::ThreadPool, protocol::h1::handle_incoming};

use super::runtime::Runtime;

pub struct ThreadPooledRuntime(ThreadPool);

impl ThreadPooledRuntime {
    pub fn new() -> std::io::Result<Self> {
        let pool = ThreadPool::new()?;
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
        listener: std::net::TcpListener,
        config: crate::server::Config,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        let thread_pool = &self.0;
        let handler = Mutex::new(Arc::new(handler));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = config.clone();
                    let handler_lock = handler.lock().expect("Failed to acquire handler lock");
                    let request_handler = handler_lock.clone();
                    thread_pool.execute(move || {
                        match handle_incoming(&request_handler, &config, stream) {
                            Ok(_) => {}
                            Err(err) => log::error!("{err}"),
                        }
                    })?;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
