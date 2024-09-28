use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

#[derive(Default)]
pub struct ThreadSpawner {
    tasks: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl ThreadSpawner {
    pub fn new() -> Self {
        ThreadSpawner {
            tasks: Default::default(),
        }
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.read().expect("Failed to acquire lock").len()
    }

    fn check_for_finished(&self) {
        self.tasks
            .write()
            .expect("Failed to acquire write lock")
            .retain(|handle| !handle.is_finished());
    }

    pub fn spawn<F: FnOnce() + Send + 'static>(
        &self,
        name: Option<String>,
        stack_size: Option<usize>,
        f: F,
    ) -> std::io::Result<()> {
        self.check_for_finished();
        let mut builder = std::thread::Builder::new();

        if let Some(name) = name {
            builder = builder.name(name);
        }

        if let Some(stack_size) = stack_size {
            builder = builder.stack_size(stack_size);
        }

        let join_handle = builder.spawn(f)?;

        self.tasks
            .write()
            .expect("Failed to acquire write lock")
            .push(join_handle);

        Ok(())
    }

    pub fn join(&self) -> std::io::Result<()> {
        let mut tasks = self.tasks.write().expect("Failed to acquire lock");
        let join_handles = tasks.drain(..);

        for handle in join_handles {
            handle
                .join()
                .map_err(|_| std::io::Error::other("Failed to join thread"))?;
        }

        Ok(())
    }
}
