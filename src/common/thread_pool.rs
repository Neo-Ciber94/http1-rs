use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::AtomicUsize,
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

type Task = Box<dyn FnOnce() + Send + Sync + 'static>;

#[derive(Clone)]
struct WorkerTaskCount(Arc<AtomicUsize>);

impl WorkerTaskCount {
    pub fn get(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Acquire);
    }

    pub fn decrement(&self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Acquire);
    }
}

struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn new(
        name: Option<String>,
        stack_size: Option<usize>,
        receiver: Arc<Mutex<Receiver<Task>>>,
        task_count: WorkerTaskCount,
    ) -> std::io::Result<Self> {
        let mut builder = std::thread::Builder::new();

        if let Some(name) = name {
            builder = builder.name(name);
        }

        if let Some(stack_size) = stack_size {
            builder = builder.stack_size(stack_size);
        }

        let handle = builder.spawn(move || {
            let lock = receiver.lock().expect("Failed to get task receiver lock");
            while let Ok(task) = lock.recv() {
                task();

                // Decrement task count
                task_count.decrement();
            }
        })?;

        Ok(Worker { handle })
    }
}

impl Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("handle", &self.handle)
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HandleId(usize);
impl HandleId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Acquire);
        HandleId(id)
    }
}
type AdditionalHandleMap = Arc<Mutex<HashMap<HandleId, JoinHandle<()>>>>;

pub struct ThreadPool {
    name: Option<String>,
    stack_size: Option<usize>,
    workers: Vec<Worker>,
    sender: Sender<Task>,
    task_count: WorkerTaskCount,
    spawn_on_full: bool,
    additional_handles: AdditionalHandleMap,
}

impl ThreadPool {
    pub fn new() -> std::io::Result<Self> {
        let parallelism = std::thread::available_parallelism()?;
        Self::with_workers(parallelism.get())
    }

    pub fn with_workers(worker_count: usize) -> std::io::Result<Self> {
        Builder::new().worker_count(worker_count).build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn stack_size(&self) -> Option<usize> {
        self.stack_size
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn pending_count(&self) -> usize {
        self.task_count.get()
    }

    pub fn enqueue<F: FnOnce() + Send + Sync + 'static>(&mut self, f: F) -> std::io::Result<()> {
        // All workers are in use, we spawn a new thread
        if self.spawn_on_full && self.pending_count() > self.worker_count() {
            return self.spawn_additional_task(f);
        }

        self.task_count.increment();
        let task = Box::new(f);
        self.sender
            .send(task)
            .map_err(|err| std::io::Error::other(err))
    }

    fn spawn_additional_task<F: FnOnce() + Send + Sync + 'static>(
        &mut self,
        f: F,
    ) -> std::io::Result<()> {
        let mut builder = std::thread::Builder::new();

        if let Some(name) = self.name.clone() {
            builder = builder.name(name);
        }

        if let Some(stack_size) = self.stack_size {
            builder = builder.stack_size(stack_size);
        }

        let handle_id = HandleId::next();
        let additional_handles = self.additional_handles.clone();

        let handle = {
            let h = additional_handles.clone();
            builder.spawn(move || {
                f();

                h.lock()
                    .expect("Failed to get lock for additional handles")
                    .remove(&handle_id);
            })?
        };

        additional_handles
            .lock()
            .map_err(|_| std::io::Error::other("Failed to lock additional handles lock"))?
            .insert(handle_id, handle);

        return Ok(());
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let workers = std::mem::take(&mut self.workers);

        for worker in workers {
            let _ = worker.handle.join();
        }
    }
}

pub struct Builder {
    name: Option<String>,
    stack_size: Option<usize>,
    worker_count: usize,
    spawn_on_full: bool,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            name: None,
            stack_size: None,
            worker_count: 4,
            spawn_on_full: false,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = Some(stack_size);
        self
    }

    pub fn spawn_on_full(mut self, spawn: bool) -> Self {
        self.spawn_on_full = spawn;
        self
    }

    pub fn worker_count(mut self, worker_count: usize) -> Self {
        assert!(worker_count > 0, "Worker count must be greater than 0");
        self.worker_count = worker_count;
        self
    }

    pub fn build(self) -> std::io::Result<ThreadPool> {
        let Self {
            name,
            stack_size,
            worker_count,
            spawn_on_full,
        } = self;

        let (sender, receiver) = channel::<Task>();
        let mut workers = Vec::with_capacity(worker_count);

        let task_count = WorkerTaskCount(Default::default());
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..worker_count {
            let worker = Worker::new(
                name.clone(),
                stack_size,
                receiver.clone(),
                task_count.clone(),
            )?;
            workers.push(worker);
        }

        Ok(ThreadPool {
            name,
            workers,
            sender,
            task_count,
            spawn_on_full,
            stack_size,
            additional_handles: Default::default(),
        })
    }
}
