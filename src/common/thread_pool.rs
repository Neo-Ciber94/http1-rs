use std::{
    any::Any,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex, TryLockError,
    },
    thread::JoinHandle,
};

use super::thread_spawner::ThreadSpawner;

type Task = Box<dyn FnOnce() + Send + Sync + 'static>;

#[derive(Clone)]
struct WorkerTaskCount(Arc<AtomicUsize>);

impl WorkerTaskCount {
    pub fn get(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn increment(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }

    pub fn decrement(&self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

struct WorkerTaskCountGuard(WorkerTaskCount);
impl Drop for WorkerTaskCountGuard {
    fn drop(&mut self) {
        self.0.decrement();
    }
}

struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn new(
        name: Option<String>,
        stack_size: Option<usize>,
        is_terminated: Arc<AtomicBool>,
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
            while !is_terminated.load(std::sync::atomic::Ordering::Acquire) {
                match receiver.try_lock() {
                    Ok(lock) => match lock.try_recv() {
                        Ok(task) => {
                            let _guard = WorkerTaskCountGuard(task_count.clone());
                            task();
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => break,
                    },
                    Err(TryLockError::Poisoned(err)) => panic!("{err}"),
                    Err(TryLockError::WouldBlock) => {}
                }
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

#[derive(Debug)]
pub enum TerminateError {
    JoinError(Box<dyn Any + 'static>),
    Other(String),
}

pub struct ThreadPool {
    name: Option<String>,
    stack_size: Option<usize>,
    workers: Vec<Worker>,
    sender: Sender<Task>,
    task_count: WorkerTaskCount,
    spawn_on_full: bool,
    is_terminated: Arc<AtomicBool>,
    additional_tasks_spawner: ThreadSpawner,
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

    pub fn additional_task_count(&self) -> usize {
        self.additional_tasks_spawner.pending_count()
    }

    pub fn is_terminated(&self) -> bool {
        self.is_terminated
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn enqueue<F: FnOnce() + Send + Sync + 'static>(&mut self, f: F) -> std::io::Result<()> {
        if self.is_terminated() {
            return Err(std::io::Error::other("ThreadPool was terminated"));
        }

        // All workers are in use, we spawn a new thread
        if self.spawn_on_full && self.pending_count() > self.worker_count() {
            let name = self.name.clone();
            let stack_size = self.stack_size.clone();
            return self
                .additional_tasks_spawner
                .spawn(name, stack_size, f)
                .map(|_| ());
        }

        self.task_count.increment();
        let task = Box::new(f);
        self.sender
            .send(task)
            .map_err(|err| std::io::Error::other(err))
    }

    pub fn terminate(&mut self) -> Result<(), TerminateError> {
        self.is_terminated
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let workers = std::mem::take(&mut self.workers);
        for w in workers {
            w.handle
                .join()
                .map_err(|err| TerminateError::JoinError(err))?;
        }

        self.additional_tasks_spawner
            .join_all()
            .map_err(|_| TerminateError::Other("Failed to join additional tasks".into()))?;
        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.terminate().expect("Failed to drop ThreadPool")
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
        let is_terminated = Arc::new(AtomicBool::new(false));

        for _ in 0..worker_count {
            let worker = Worker::new(
                name.clone(),
                stack_size,
                is_terminated.clone(),
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
            is_terminated,
            additional_tasks_spawner: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{atomic::AtomicBool, Arc},
        time::Duration,
    };

    use super::ThreadPool;

    #[derive(Clone)]
    struct Work(Arc<AtomicBool>);
    impl Work {
        pub fn work(&self) {
            while !self.0.load(std::sync::atomic::Ordering::Relaxed) {
                println!("working...");
                std::thread::yield_now();
            }
        }
    }

    #[test]
    fn should_drop_thread_pool() {
        let pool = ThreadPool::with_workers(2).unwrap();
        drop(pool);
    }

    #[test]
    fn should_enqueue_tasks() {
        let mut pool = ThreadPool::with_workers(2).unwrap();
        pool.enqueue(|| {}).unwrap();
        pool.enqueue(|| {}).unwrap();
        pool.enqueue(|| {}).unwrap();
    }

    #[test]
    fn should_block_until_all_worker_finish() {
        let mut pool = ThreadPool::with_workers(2).unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.enqueue(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.is_terminated(), false);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        is_done.store(true, std::sync::atomic::Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(10)); // Wait until all the tasks finish

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_wait_before_push_more_tasks() {
        let mut pool = ThreadPool::with_workers(2).unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.enqueue(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.is_terminated(), false);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            let w = work.clone();
            pool.enqueue(move || w.work()).unwrap();
        }

        assert_eq!(pool.pending_count(), 3);

        is_done.store(true, std::sync::atomic::Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(10)); // Wait until all the tasks finish

        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_spawn_additional_threads_if_needed() {
        let mut pool = ThreadPool::builder()
            .worker_count(2)
            .spawn_on_full(true)
            .build()
            .unwrap();

        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.enqueue(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.is_terminated(), false);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            for _ in 0..3 {
                let w = work.clone();
                pool.enqueue(move || w.work()).unwrap();
            }
        }

        // assert_eq!(pool.pending_count(), 2);
        // assert_eq!(pool.additional_task_count(), 3);

        // is_done.store(true, std::sync::atomic::Ordering::Relaxed);
        // std::thread::sleep(Duration::from_millis(10)); // Wait until all the tasks finish

        // assert_eq!(pool.pending_count(), 0);
        // assert_eq!(pool.additional_task_count(), 0);
    }
}
