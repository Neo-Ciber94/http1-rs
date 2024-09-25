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
    is_active: Arc<AtomicBool>,
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
        struct IsActiveGuard(Arc<AtomicBool>);
        impl Drop for IsActiveGuard {
            fn drop(&mut self) {
                self.0.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let mut builder = std::thread::Builder::new();

        if let Some(name) = name {
            builder = builder.name(name);
        }

        if let Some(stack_size) = stack_size {
            builder = builder.stack_size(stack_size);
        }

        let is_active = Arc::new(AtomicBool::new(false));
        let handle = {
            let is_active = is_active.clone();

            builder.spawn(move || {
                while !is_terminated.load(std::sync::atomic::Ordering::Acquire) {
                    match receiver.try_lock() {
                        Ok(lock) => match lock.try_recv() {
                            Ok(task) => {
                                is_active.store(true, std::sync::atomic::Ordering::Relaxed);
                                let _is_active_guard = IsActiveGuard(is_active.clone());
                                let _work_count_guard = WorkerTaskCountGuard(task_count.clone());

                                // Execute the task
                                task();
                            }
                            Err(TryRecvError::Empty) => {}
                            Err(TryRecvError::Disconnected) => break,
                        },
                        Err(TryLockError::Poisoned(err)) => panic!("{err}"),
                        Err(TryLockError::WouldBlock) => {}
                    }
                }
            })?
        };

        Ok(Worker { handle, is_active })
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(std::sync::atomic::Ordering::Relaxed)
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

struct State {
    name: Option<String>,
    stack_size: Option<usize>,
    workers: Vec<Worker>,
    sender: Sender<Task>,
    task_count: WorkerTaskCount,
    spawn_on_full: bool,
    is_terminated: Arc<AtomicBool>,
    additional_tasks_spawner: ThreadSpawner,
}

#[derive(Clone)]
pub struct ThreadPool {
    inner: Arc<State>,
}

impl ThreadPool {
    pub fn new() -> std::io::Result<Self> {
        let parallelism = std::thread::available_parallelism()?;
        Self::with_workers(parallelism.get())
    }

    pub fn with_workers(num_threads: usize) -> std::io::Result<Self> {
        Builder::new().num_threads(num_threads).build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    pub fn stack_size(&self) -> Option<usize> {
        self.inner.stack_size
    }

    pub fn worker_count(&self) -> usize {
        self.inner.workers.len()
    }

    pub fn pending_count(&self) -> usize {
        self.inner.task_count.get()
    }

    pub fn additional_task_count(&self) -> usize {
        self.inner.additional_tasks_spawner.pending_count()
    }

    pub fn is_terminated(&self) -> bool {
        self.inner
            .is_terminated
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn execute<F: FnOnce() + Send + Sync + 'static>(&self, f: F) -> std::io::Result<()> {
        if self.is_terminated() {
            return Err(std::io::Error::other("ThreadPool was terminated"));
        }

        // All workers are in use, we spawn a new thread
        if self.inner.spawn_on_full && self.pending_count() > self.worker_count() {
            let name = self.inner.name.clone();
            let stack_size = self.inner.stack_size.clone();
            return self
                .inner
                .additional_tasks_spawner
                .spawn(name, stack_size, f)
                .map(|_| ());
        }

        self.inner.task_count.increment();
        let task = Box::new(f);
        self.inner
            .sender
            .send(task)
            .map_err(|err| std::io::Error::other(err))
    }

    pub fn join(&self) -> Result<(), TerminateError> {
        self.inner
            .is_terminated
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let workers = &self.inner.workers;

        // Wait until all workers have not work again
        loop {
            let all_idle = workers.iter().all(|w| !w.is_active());

            if all_idle {
                break;
            }
        }

        self.inner
            .additional_tasks_spawner
            .join()
            .map_err(|_| TerminateError::Other("Failed to join additional tasks".into()))?;
        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.join().expect("Failed to drop ThreadPool")
    }
}

pub struct Builder {
    name: Option<String>,
    stack_size: Option<usize>,
    num_threads: usize,
    spawn_on_full: bool,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            name: None,
            stack_size: None,
            num_threads: 4,
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

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        assert!(num_threads > 0, "Number of threads must be greater than 0");
        self.num_threads = num_threads;
        self
    }

    pub fn build(self) -> std::io::Result<ThreadPool> {
        let Self {
            name,
            stack_size,
            num_threads,
            spawn_on_full,
        } = self;

        let (sender, receiver) = channel::<Task>();
        let mut workers = Vec::with_capacity(num_threads);

        let task_count = WorkerTaskCount(Default::default());
        let receiver = Arc::new(Mutex::new(receiver));
        let is_terminated = Arc::new(AtomicBool::new(false));

        for _ in 0..num_threads {
            let worker = Worker::new(
                name.clone(),
                stack_size,
                is_terminated.clone(),
                receiver.clone(),
                task_count.clone(),
            )?;

            workers.push(worker);
        }

        let inner = Arc::new(State {
            name,
            workers,
            sender,
            task_count,
            spawn_on_full,
            stack_size,
            is_terminated,
            additional_tasks_spawner: Default::default(),
        });

        Ok(ThreadPool { inner })
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
            println!("working...");
            while !self.0.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::yield_now();
            }

            println!("work done!");
        }
    }

    #[test]
    fn should_drop_thread_pool() {
        let pool = ThreadPool::with_workers(2).unwrap();
        drop(pool);
    }

    #[test]
    fn should_enqueue_tasks() {
        let pool = ThreadPool::with_workers(2).unwrap();
        pool.execute(|| {}).unwrap();
        pool.execute(|| {}).unwrap();
        pool.execute(|| {}).unwrap();
    }

    #[test]
    fn should_block_until_all_worker_finish() {
        let pool = ThreadPool::with_workers(2).unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
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
        let pool = ThreadPool::with_workers(2).unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.is_terminated(), false);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            let w = work.clone();
            pool.execute(move || w.work()).unwrap();
        }

        assert_eq!(pool.pending_count(), 3);

        is_done.store(true, std::sync::atomic::Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(10)); // Wait until all the tasks finish

        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_spawn_additional_threads_if_needed() {
        let pool = ThreadPool::builder()
            .num_threads(2)
            .spawn_on_full(true)
            .build()
            .unwrap();

        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.is_terminated(), false);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            for _ in 0..3 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.pending_count(), 2);
        assert_eq!(pool.additional_task_count(), 3);

        is_done.store(true, std::sync::atomic::Ordering::Relaxed);
        // std::thread::sleep(Duration::from_millis(10)); // Wait until all the tasks finish

        // assert_eq!(pool.pending_count(), 0);
        // assert_eq!(pool.additional_task_count(), 0);
    }
}
