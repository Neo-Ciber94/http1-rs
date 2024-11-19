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
        self.cond_var.notify_one();
        Ok(())
    }

    fn recv(&self, timeout: Option<Duration>) -> Result<T, RecvError> {
        self.check_connected()?;

        let mut lock = match self.queue.lock() {
            Ok(x) => x,
            Err(_) => {
                return Err(RecvError::NotReceived);
            }
        };

        loop {
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

                return Ok(value);
            }

            lock = if let Some(dur) = timeout {
                let (ret, _) = self
                    .cond_var
                    .wait_timeout(lock, dur)
                    .map_err(|_| RecvError::NotReceived)?;

                ret
            } else {
                self.cond_var
                    .wait(lock)
                    .map_err(|_| RecvError::NotReceived)?
            }
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

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.channel
            .receivers_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            channel: self.channel.clone(),
        }
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
        sync::{atomic::AtomicUsize, Arc, Mutex},
        time::Duration,
    };

    use super::channel;

    #[test]
    fn should_send_value() {
        let (sender, receiver) = channel::<&str>();

        let mut r1 = receiver.clone();
        let mut r2 = receiver.clone();

        let values = Arc::new(Mutex::new(vec![]));
        let signal = Arc::new(AtomicUsize::new(0));

        {
            let values = values.clone();
            let s = signal.clone();

            std::thread::spawn(move || {
                values
                    .lock()
                    .unwrap()
                    .push(r1.recv_timeout(Duration::from_millis(100)).unwrap());
                values
                    .lock()
                    .unwrap()
                    .push(r1.recv_timeout(Duration::from_millis(100)).unwrap());
                s.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            })
        };

        {
            let values = values.clone();
            let s = signal.clone();

            std::thread::spawn(move || {
                values
                    .lock()
                    .unwrap()
                    .push(r2.recv_timeout(Duration::from_millis(100)).unwrap());
                values
                    .lock()
                    .unwrap()
                    .push(r2.recv_timeout(Duration::from_millis(100)).unwrap());
                s.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            })
        };

        sender.send("Fionna").unwrap();
        sender.send("Cake").unwrap();

        // while signal.load(std::sync::atomic::Ordering::Relaxed) < 2 {}
        std::thread::sleep(Duration::from_millis(100));

        let v = &*values.lock().unwrap();

        assert_eq!(v.len(), 4);
        assert_eq!(v.iter().filter(|x| **x == "Fionna").count(), 2);
        assert_eq!(v.iter().filter(|x| **x == "Cake").count(), 2);
    }
}
