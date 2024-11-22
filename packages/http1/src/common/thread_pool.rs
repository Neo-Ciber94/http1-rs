use std::{
    any::Any,
    fmt::Debug,
    sync::{
        atomic::AtomicUsize,
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
};

type Task = Box<dyn FnOnce() + Send + 'static>;

const DEFAULT_WORKER_NAME: &str = "thread_pool_worker";

#[derive(Default)]
struct AtomicCounter(AtomicUsize);
impl AtomicCounter {
    pub fn get(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn increment(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub enum TerminateError {
    JoinError(Box<dyn Any + 'static>),
    Other(String),
}

pub struct Monitoring {
    active_worker_count: AtomicCounter,
    pending_tasks_count: AtomicCounter,
    panicked_worker_count: AtomicCounter,
}

struct State {
    name: String,
    stack_size: Option<usize>,
    sender: Sender<Task>,
    receiver: Arc<Mutex<Receiver<Task>>>,
    on_worker_panic: Option<fn()>,
    monitoring: Arc<Monitoring>,
    max_workers_count: Option<usize>,
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

    pub fn with_workers(num_workers: usize) -> std::io::Result<Self> {
        Builder::new().num_workers(num_workers).build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Returns the name used as prefix for the worker threads.
    pub fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    /// Returns the size of the stack of the worker threads.
    pub fn stack_size(&self) -> Option<usize> {
        self.inner.stack_size
    }

    /// Returns the number of active worker threads.
    pub fn worker_count(&self) -> usize {
        self.inner.monitoring.active_worker_count.get()
    }

    /// Returns the max allowed number of workers
    pub fn max_workers_count(&self) -> Option<usize> {
        self.inner.max_workers_count
    }

    /// Returns the number of tasks that still running.
    pub fn pending_count(&self) -> usize {
        self.inner.monitoring.pending_tasks_count.get()
    }

    /// Returns the number of workers that panicked.
    pub fn panicked_count(&self) -> usize {
        self.inner.monitoring.panicked_worker_count.get()
    }

    /// Executes the given task on an available worker.
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) -> std::io::Result<()> {
        let is_full = self.pending_count() >= self.worker_count();

        if is_full {
            match self.inner.max_workers_count {
                Some(max_workers_count) => {
                    if self.worker_count() < max_workers_count {
                        let _ = spawn_worker(&self.inner);
                    }
                }
                None => {
                    let _ = spawn_worker(&self.inner);
                }
            }
        }

        // Increment the count
        self.inner.monitoring.pending_tasks_count.increment();

        // Spawn the new task
        let task = Box::new(f);
        self.inner
            .sender
            .send(task)
            .expect("Failed to send a task to the ThreadPool");

        Ok(())
    }

    pub fn join(&self) -> Result<(), TerminateError> {
        // Wait for all pending tasks to finish
        while self.inner.monitoring.pending_tasks_count.get() > 0 {
            std::thread::yield_now()
        }

        Ok(())
    }
}

pub struct Builder {
    name: Option<String>,
    stack_size: Option<usize>,
    num_workers: usize,
    max_workers_count: Option<usize>,
    on_worker_panic: Option<fn()>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            name: None,
            stack_size: None,
            num_workers: 4,
            max_workers_count: None,
            on_worker_panic: None,
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

    pub fn num_workers(mut self, num_workers: usize) -> Self {
        assert!(num_workers > 0, "Number of threads must be greater than 0");
        self.num_workers = num_workers;
        self
    }

    pub fn max_workers_count(mut self, max_workers_count: Option<usize>) -> Self {
        self.max_workers_count = max_workers_count;
        self
    }

    pub fn on_worker_panic(mut self, on_panic: fn()) -> Self {
        self.on_worker_panic = Some(on_panic);
        self
    }

    pub fn build(self) -> std::io::Result<ThreadPool> {
        let Self {
            name,
            stack_size,
            num_workers,
            max_workers_count,
            on_worker_panic,
        } = self;

        let (sender, receiver) = channel::<Task>();
        let receiver = Arc::new(Mutex::new(receiver));

        let monitoring = Arc::new(Monitoring {
            active_worker_count: AtomicCounter::default(),
            panicked_worker_count: AtomicCounter::default(),
            pending_tasks_count: AtomicCounter::default(),
        });

        let name = name.unwrap_or_else(|| String::from(DEFAULT_WORKER_NAME));

        let inner = Arc::new(State {
            name,
            sender,
            stack_size,
            monitoring,
            on_worker_panic,
            max_workers_count,
            receiver: receiver.clone(),
        });

        for _ in 0..num_workers {
            spawn_worker(&inner)?;
        }

        Ok(ThreadPool { inner })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

fn spawn_worker(state: &Arc<State>) -> std::io::Result<()> {
    struct PendingTasksGuard(Arc<Monitoring>);
    impl Drop for PendingTasksGuard {
        fn drop(&mut self) {
            self.0.pending_tasks_count.decrement();
        }
    }

    struct ActiveWorkerGuard(Arc<Monitoring>);
    impl Drop for ActiveWorkerGuard {
        fn drop(&mut self) {
            self.0.active_worker_count.decrement();
        }
    }

    let mut builder = std::thread::Builder::new();

    let worker_name = format!(
        "{worker_name}#{worker_count}",
        worker_name = state.name.as_str(),
        worker_count = state.monitoring.active_worker_count.get() + 1
    );

    builder = builder.name(worker_name);

    if let Some(stack_size) = state.stack_size {
        builder = builder.stack_size(stack_size);
    }

    // Add a new active worker
    state.monitoring.active_worker_count.increment();

    let state = Arc::clone(state);

    builder.spawn(move || {
        let _guard_active_worker = ActiveWorkerGuard(state.monitoring.clone());

        loop {
            match state.receiver.lock() {
                Ok(rx) => match rx.try_recv() {
                    Ok(task) => {
                        let _pending_task_guard = PendingTasksGuard(state.monitoring.clone());

                        // Execute the task
                        task();
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                },
                Err(_) => {
                    state.monitoring.panicked_worker_count.increment();

                    if let Some(on_panic) = state.on_worker_panic {
                        // We sure ensure this do not panic because it will prevent other worker from spawning
                        let _ = std::panic::catch_unwind(on_panic);
                    }

                    // Spawn a new worker if this thread is poisoned
                    spawn_worker(&state).expect("failed to spawn worker after previous one failed");

                    // Exits
                    break;
                }
            }
        }
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{atomic::AtomicBool, Arc},
        time::{Duration, Instant},
    };

    use super::ThreadPool;

    #[derive(Clone)]
    struct Work(Arc<AtomicBool>);
    impl Work {
        pub fn work(&self) {
            while !self.0.load(std::sync::atomic::Ordering::Relaxed) {
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
    fn should_execute_tasks() {
        let pool = ThreadPool::with_workers(2).unwrap();
        pool.execute(|| {}).unwrap();
        pool.execute(|| {}).unwrap();
        pool.execute(|| {}).unwrap();
    }

    #[test]
    fn should_block_until_all_worker_finish() {
        let pool = ThreadPool::with_workers(2).unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 0);

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        is_done.store(true, std::sync::atomic::Ordering::Release);
        pool.join().unwrap();
        std::thread::sleep(Duration::from_millis(50));

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_wait_before_push_more_tasks() {
        let pool = ThreadPool::builder()
            .num_workers(2)
            .max_workers_count(Some(2))
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

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            let w = work.clone();
            pool.execute(move || w.work()).unwrap();
        }

        assert_eq!(pool.pending_count(), 3);

        is_done.store(true, std::sync::atomic::Ordering::Release);
        pool.join().unwrap();
        std::thread::sleep(Duration::from_millis(50));

        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_spawn_additional_threads_if_needed() {
        let pool = ThreadPool::builder().num_workers(2).build().unwrap();

        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.pending_count(), 2);

        {
            for _ in 0..3 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        assert_eq!(pool.worker_count(), 5);

        is_done.store(true, std::sync::atomic::Ordering::Release);
        pool.join().unwrap();

        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_spawn_additional_worker_on_panic() {
        let pool = ThreadPool::builder().num_workers(2).build().unwrap();
        let is_done = Arc::new(AtomicBool::new(false));

        let work = Work(is_done.clone());
        {
            for _ in 0..2 {
                let w = work.clone();
                pool.execute(move || w.work()).unwrap();
            }
        }

        pool.execute(|| panic!("Oh oh")).unwrap();

        assert_eq!(pool.pending_count(), 3);
        assert_eq!(pool.panicked_count(), 0);

        is_done.store(true, std::sync::atomic::Ordering::Release);
        pool.join().unwrap();

        let now = Instant::now();
        while pool.panicked_count() == 0 && now.elapsed() < Duration::from_millis(1000) {}

        assert_eq!(pool.pending_count(), 0);
        assert_eq!(pool.panicked_count(), 1);
    }
}
