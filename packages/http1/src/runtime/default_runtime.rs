use crate::{common::thread_pool::ThreadPool, handler::RequestHandler};

use super::{
    runtime::{Runtime, StartRuntime},
    thread_pooled_runtime::ThreadPooledRuntime,
};

pub struct DefaultRuntime(ThreadPooledRuntime);

impl Default for DefaultRuntime {
    fn default() -> Self {
        let pool = ThreadPool::builder()
            .name("default_runtime")
            .spawn_on_full(true)
            .build()
            .expect("Failed to create default runtime ThreadPool");

        DefaultRuntime(ThreadPooledRuntime::with_pool(pool))
    }
}

impl Runtime for DefaultRuntime {
    type Output = ();

    fn start<H: RequestHandler + Send + Sync + 'static>(
        self,
        args: StartRuntime,
        handler: H,
    ) -> std::io::Result<Self::Output> {
        self.0.start(args, handler)
    }
}
