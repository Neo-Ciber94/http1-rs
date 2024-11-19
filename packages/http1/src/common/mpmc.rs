use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{atomic::AtomicUsize, Arc, Condvar, Mutex},
    time::Duration,
};

struct Slot<T> {
    value: T,
    received: usize,
}

struct Channel<T> {
    queue: Mutex<VecDeque<Slot<T>>>,
    receivers_count: AtomicUsize,
    senders_count: AtomicUsize,
    cond_var: Condvar,
}

impl<T> Channel<T>
where
    T: Clone,
{
    fn check_connected(&self) -> Result<(), RecvError> {
        if self
            .senders_count
            .load(std::sync::atomic::Ordering::Relaxed)
            == 0
        {
            return Err(RecvError::Disconnected);
        }

        Ok(())
    }

    fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut lock = match self.queue.lock() {
            Ok(x) => x,
            Err(_) => {
                return Err(SendError(value));
            }
        };

        lock.push_back(Slot { value, received: 0 });
        self.cond_var.notify_all();
        Ok(())
    }

    fn recv(&self, timeout: Option<Duration>) -> Result<T, RecvError> {
        let mut lock = match self.queue.try_lock() {
            Ok(x) => x,
            Err(_) => {
                return Err(RecvError::NotReceived);
            }
        };

        // Wait for new data
        lock = if let Some(dur) = timeout {
            let (lock, timeout_result) = self
                .cond_var
                .wait_timeout_while(lock, dur, |x| x.is_empty())
                .map_err(|_| RecvError::NotReceived)?;

            if timeout_result.timed_out() {
                return Err(RecvError::Timeout);
            }

            lock
        } else {
            self.cond_var
                .wait_while(lock, |x| x.is_empty())
                .map_err(|_| RecvError::NotReceived)?
        };

        self.check_connected()?;

        if let Some(slot) = lock.front_mut() {
            let receivers_count = self
                .receivers_count
                .load(std::sync::atomic::Ordering::Relaxed);

            slot.received += 1;

            let value = if slot.received >= receivers_count {
                lock.pop_front().unwrap().value
            } else {
                slot.value.clone()
            };

            Ok(value)
        } else {
            Err(RecvError::NotReceived)
        }
    }
}

pub struct SendError<T>(T);

impl<T> Debug for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendError").finish_non_exhaustive()
    }
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T>
where
    T: Clone,
{
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.channel.send(value)
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.channel
            .receivers_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Receiver {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.channel
            .senders_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.channel
            .senders_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        self.channel.cond_var.notify_all();
    }
}

#[derive(Debug)]
pub enum RecvError {
    Disconnected,
    NotReceived,
    Timeout,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T: Clone> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, RecvError> {
        self.channel.recv(None)
    }

    pub fn recv_timeout(&mut self, dur: Duration) -> Result<T, RecvError> {
        self.channel.recv(Some(dur))
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.channel
            .receivers_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

pub fn channel<T: Clone>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        queue: Mutex::new(VecDeque::new()),
        receivers_count: AtomicUsize::new(0),
        senders_count: AtomicUsize::new(1),
        cond_var: Condvar::new(),
    });

    let sender = Sender {
        channel: Arc::clone(&channel),
    };

    let receiver = Receiver { channel };

    (sender, receiver)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier, Mutex},
        time::Duration,
    };

    use super::channel;

    #[test]
    fn should_send_value() {
        let (sender, mut receiver) = channel::<&str>();

        let mut r2 = sender.subscribe();

        let values = Arc::new(Mutex::new(vec![]));
        let barrier = Arc::new(Barrier::new(2));

        let t1 = {
            let values = values.clone();
            let barrier = barrier.clone();

            std::thread::spawn(move || {
                values.lock().unwrap().push(receiver.recv().unwrap());
                values.lock().unwrap().push(receiver.recv().unwrap());

                barrier.wait();
            })
        };

        let t2 = {
            let values = values.clone();

            std::thread::spawn(move || {
                values.lock().unwrap().push(r2.recv().unwrap());
                values.lock().unwrap().push(r2.recv().unwrap());
                barrier.wait();
            })
        };

        sender.send("Fionna").unwrap();
        sender.send("Cake").unwrap();

        // Wait for the thread
        t1.join().unwrap();
        t2.join().unwrap();

        let v = &*values.lock().unwrap();

        assert_eq!(v.len(), 4);
        assert_eq!(v.iter().filter(|x| **x == "Fionna").count(), 2);
        assert_eq!(v.iter().filter(|x| **x == "Cake").count(), 2);
    }

    #[test]
    fn should_timeout_on_recv() {
        let (_, mut receiver) = channel::<&str>();
        let result = receiver.recv_timeout(Duration::from_millis(100));
        assert!(matches!(result, Err(super::RecvError::Timeout)));
    }

    #[test]
    fn should_disconnect_on_drop() {
        let (sender, mut receiver) = channel::<&str>();
        drop(sender); // Drop the sender
        let result = receiver.recv();
        assert!(matches!(result, Err(super::RecvError::Disconnected)));
    }

    #[test]
    fn should_receive_ordered_values() {
        let (sender, mut receiver) = channel::<&str>();
        sender.send("Jake").unwrap();
        sender.send("Finn").unwrap();
        sender.send("Ice King").unwrap();

        assert_eq!(receiver.recv().unwrap(), "Jake");
        assert_eq!(receiver.recv().unwrap(), "Finn");
        assert_eq!(receiver.recv().unwrap(), "Ice King");
    }

    #[test]
    fn should_support_multiple_subscribers() {
        let (sender, mut receiver1) = channel::<&str>();
        let mut receiver2 = sender.subscribe();

        sender.send("Princess Bubblegum").unwrap();
        sender.send("Marceline").unwrap();

        assert_eq!(receiver1.recv().unwrap(), "Princess Bubblegum");
        assert_eq!(receiver1.recv().unwrap(), "Marceline");
        assert_eq!(receiver2.recv().unwrap(), "Princess Bubblegum");
        assert_eq!(receiver2.recv().unwrap(), "Marceline");
    }

    #[test]
    fn should_handle_concurrent_sends() {
        let (sender, mut receiver) = channel::<&str>();
        let barrier = Arc::new(Barrier::new(3)); // Two senders + main thread

        let t1 = {
            let sender = sender.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                sender.send("BMO").unwrap();
                barrier.wait();
            })
        };

        let t2 = {
            let sender = sender.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                sender.send("Lumpy Space Princess").unwrap();
                barrier.wait();
            })
        };

        barrier.wait(); // Wait for all threads to complete
        t1.join().unwrap();
        t2.join().unwrap();

        let received: Vec<_> = (0..2).map(|_| receiver.recv().unwrap()).collect();
        assert!(received.contains(&"BMO"));
        assert!(received.contains(&"Lumpy Space Princess"));
    }
}
