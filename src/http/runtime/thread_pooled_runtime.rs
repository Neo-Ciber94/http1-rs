use std::{
    cell::{Ref, RefCell},
    sync::{Arc, Mutex},
};

use crate::{common::thread_pool::ThreadPool, http::protocol::h1::handle_incoming};

use super::runtime::Runtime;

pub struct ThreadPooledRuntime(RefCell<ThreadPool>);

impl ThreadPooledRuntime {
    pub fn new() -> std::io::Result<Self> {
        let pool = ThreadPool::new()?;
        Ok(Self::with_pool(pool))
    }

    pub fn with_pool(pool: ThreadPool) -> Self {
        ThreadPooledRuntime(RefCell::new(pool))
    }

    pub fn pool(&self) -> Ref<ThreadPool> {
        self.0.borrow()
    }
}

impl Runtime for ThreadPooledRuntime {
    type Output = ();

    fn start<H: crate::handler::RequestHandler + Send + Sync + 'static>(
        self,
        listener: std::net::TcpListener,
        config: crate::server::ServerConfig,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        let mut thread_pool = self.0.borrow_mut();
        let handler = Mutex::new(Arc::new(handler));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = config.clone();
                    let handler_lock = handler.lock().expect("Failed to acquire handler lock");
                    let request_handler = handler_lock.clone();
                    thread_pool.spawn(move || handle_incoming(request_handler, config, stream))?;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
