use std::net::TcpListener;

use crate::{common::thread_pool::ThreadPool, handler::RequestHandler, server::ServerConfig};

use super::{runtime::Runtime, thread_pooled_runtime::ThreadPooledRuntime};

pub struct DefaultRuntime(ThreadPooledRuntime);

impl Default for DefaultRuntime {
    fn default() -> Self {
        let pool = ThreadPool::builder()
            .name("default_runtime")
            .spawn_on_full(false) // should be true
            .build()
            .expect("Failed to create default runtime ThreadPool");

        DefaultRuntime(ThreadPooledRuntime::with_pool(pool))
    }
}

impl Runtime for DefaultRuntime {
    type Output = ();

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        listener: TcpListener,
        config: ServerConfig,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        self.0.start(listener, config, handler)
    }
}
