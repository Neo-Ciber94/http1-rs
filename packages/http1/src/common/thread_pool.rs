use std::sync::{
    atomic::AtomicUsize,
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
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

pub struct Monitoring {
    active_worker_count: AtomicCounter,
    pending_tasks_count: AtomicCounter,
    panicked_worker_count: AtomicCounter,
}

struct SharedState {
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
    state: Arc<SharedState>,
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
        self.state.name.as_str()
    }

    /// Returns the size of the stack of the worker threads.
    pub fn stack_size(&self) -> Option<usize> {
        self.state.stack_size
    }

    /// Returns the number of active worker threads.
    pub fn worker_count(&self) -> usize {
        self.state.monitoring.active_worker_count.get()
    }

    /// Returns the max allowed number of workers
    pub fn max_workers_count(&self) -> Option<usize> {
        self.state.max_workers_count
    }

    /// Returns the number of tasks that still running.
    pub fn pending_count(&self) -> usize {
        self.state.monitoring.pending_tasks_count.get()
    }

    /// Returns the number of workers that panicked.
    pub fn panicked_count(&self) -> usize {
        self.state.monitoring.panicked_worker_count.get()
    }

    /// Executes the given task on an available worker.
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) -> std::io::Result<()> {
        let is_full = self.pending_count() >= self.worker_count();

        if is_full {
            match self.state.max_workers_count {
                Some(max_workers_count) => {
                    if self.worker_count() < max_workers_count {
                        let _ = spawn_worker(&self.state);
                    }
                }
                None => {
                    let _ = spawn_worker(&self.state);
                }
            }
        }

        // Increment the count
        self.state.monitoring.pending_tasks_count.increment();

        // Spawn the new task
        let task = Box::new(f);
        self.state
            .sender
            .send(task)
            .expect("Failed to send a task to the ThreadPool");

        Ok(())
    }

    pub fn join(&self) {
        // Wait for all pending tasks to finish
        while self.state.monitoring.pending_tasks_count.get() > 0 {
            std::thread::yield_now()
        }
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

        let inner = Arc::new(SharedState {
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

        Ok(ThreadPool { state: inner })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

struct Watchdog<'a> {
    state: &'a Arc<SharedState>,
}

impl<'a> Watchdog<'a> {
    pub fn new(state: &'a Arc<SharedState>) -> Self {
        Watchdog { state }
    }
}

impl<'a> Drop for Watchdog<'a> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            return;
        }

        let state = &self.state;
        state.monitoring.active_worker_count.decrement();
        state.monitoring.panicked_worker_count.increment();

        if let Some(on_panic) = state.on_worker_panic {
            if let Err(err) = std::panic::catch_unwind(on_panic) {
                if cfg!(debug_assertions) {
                    panic!("`ThreadPool` on_panic hook SHOULD NOT panic: {err:?}")
                }
            }
        }

        log::error!(
            "thread `{:?}` panicked, spawning a new worker",
            std::thread::current()
        );

        if let Err(err) = spawn_worker(state) {
            log::error!("failed to spawn new thread: {err:?}")
        }
    }
}

struct CallOnDrop<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> CallOnDrop<F> {
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }
}

impl<F: FnOnce()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

fn spawn_worker(state: &Arc<SharedState>) -> std::io::Result<()> {
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
        // The watchdog will spawn a new thread if the current one panics
        let _watchdog = Watchdog::new(&state);

        loop {
            // Decrement active work count when completes
            let _guard = CallOnDrop::new({
                let monitoring = state.monitoring.clone();
                move || monitoring.pending_tasks_count.decrement()
            });

            let task = {
                let receiver = state.receiver.lock().expect("failed to lock job receiver");

                match receiver.recv() {
                    Ok(t) => t,
                    Err(..) => break,
                }
            };

            // Execute the task
            task();
        }
    })?;

    Ok(())
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
        pool.join();
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
        pool.join();
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
        pool.join();

        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn should_spawn_additional_worker_on_panic() {
        let pool = ThreadPool::builder().num_workers(3).build().unwrap();

        assert_eq!(pool.worker_count(), 3);

        pool.execute(|| panic!("Oh oh")).unwrap();
        pool.execute(|| panic!("Oh oh")).unwrap();

        assert_eq!(pool.pending_count(), 2);

        pool.join();
        std::thread::sleep(Duration::from_millis(100));

        assert_eq!(pool.worker_count(), 3);
        assert_eq!(pool.pending_count(), 0);
        assert_eq!(pool.panicked_count(), 2);
    }
}
