use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, RwLock,
    },
    thread::JoinHandle,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandleId(usize);

impl HandleId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let next_id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        HandleId(next_id)
    }
}

#[derive(Default)]
pub struct ThreadSpawner {
    tasks: Arc<RwLock<HashMap<HandleId, JoinHandle<()>>>>,
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

    pub fn spawn<F: FnOnce() + Send + 'static>(
        &mut self,
        name: Option<String>,
        stack_size: Option<usize>,
        f: F,
    ) -> std::io::Result<HandleId> {
        let mut builder = std::thread::Builder::new();

        if let Some(name) = name {
            builder = builder.name(name);
        }

        if let Some(stack_size) = stack_size {
            builder = builder.stack_size(stack_size);
        }

        let tasks = self.tasks.clone();
        let handle_id = HandleId::next();
        let is_done: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let join_handle = {
            let tasks = tasks.clone();
            let is_done = is_done.clone();

            builder.spawn(move || {
                f();

                is_done.store(true, std::sync::atomic::Ordering::Relaxed);

                let exists = tasks
                    .read()
                    .expect("Failed to acquire lock")
                    .contains_key(&handle_id);

                if exists {
                    tasks
                        .write()
                        .expect("Failed to acquire lock")
                        .remove(&handle_id);
                }
            })?
        };

        if !is_done.load(std::sync::atomic::Ordering::Acquire) {
            tasks
                .write()
                .expect("Failed to acquire lock")
                .insert(handle_id, join_handle);
        }

        Ok(handle_id)
    }

    pub fn join_all(&mut self) -> std::io::Result<()> {
        let mut tasks = self.tasks.write().expect("Failed to acquire lock");
        let entries = tasks.drain();

        for (_, join_handle) in entries {
            join_handle
                .join()
                .map_err(|_| std::io::Error::other("Failed to join thread"))?;
        }

        Ok(())
    }
}
