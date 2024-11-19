use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex, Weak},
};

type Listener<T> = Box<dyn FnMut(T) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id(usize);
impl Id {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        Id(id)
    }
}

#[derive(Debug)]
pub struct SendError;

#[derive(Debug)]
pub struct SubscribeError;

pub struct Broadcast<T> {
    listeners: Arc<Mutex<HashMap<Id, Listener<T>>>>,
}

impl<T> Clone for Broadcast<T> {
    fn clone(&self) -> Self {
        Self {
            listeners: Arc::clone(&self.listeners),
        }
    }
}

impl<T> Broadcast<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Broadcast {
            listeners: Default::default(),
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError> {
        let mut listeners = self.listeners.lock().map_err(|_| SendError)?;
        listeners
            .values_mut()
            .for_each(|listener| listener(value.clone()));
        Ok(())
    }

    pub fn subscribe<F>(&self, listener: F) -> Result<Subscription<T>, SubscribeError>
    where
        F: FnMut(T) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.lock().map_err(|_| SubscribeError)?;
        let id = Id::next();
        let weak = Arc::downgrade(&self.listeners);

        listeners.insert(id, Box::new(listener));
        Ok(Subscription { id, weak })
    }
}

#[derive(Debug)]
pub enum UnsubscribeError {
    NoConnected,
    PoisonError,
}

#[must_use]
pub struct Subscription<T> {
    id: Id,
    weak: Weak<Mutex<HashMap<Id, Listener<T>>>>,
}

impl<T> Subscription<T> {
    pub fn unsubscribe(&self) -> Result<(), UnsubscribeError> {
        let Some(listeners) = self.weak.upgrade() else {
            return Err(UnsubscribeError::NoConnected);
        };

        let mut lock = listeners
            .lock()
            .map_err(|_| UnsubscribeError::PoisonError)?;
        lock.remove(&self.id);
        Ok(())
    }

    pub fn resubscribe<F>(&mut self, listener: F) -> Result<(), SubscribeError>
    where
        F: FnMut(T) + Send + Sync + 'static,
    {
        let Some(listeners) = self.weak.upgrade() else {
            return Err(SubscribeError);
        };

        let mut lock = listeners.lock().expect("failed to get lock to unsubscribe");
        lock.insert(self.id, Box::new(listener));
        Ok(())
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_send_to_all_subscribers() {
        let broadcast = Broadcast::new();
        let received_a = Arc::new(Mutex::new(None));
        let received_b = Arc::new(Mutex::new(None));

        let received_a_clone = Arc::clone(&received_a);
        let _s = broadcast
            .subscribe(move |msg| {
                *received_a_clone.lock().unwrap() = Some(msg);
            })
            .unwrap();

        let received_b_clone = Arc::clone(&received_b);
        let _s = broadcast
            .subscribe(move |msg| {
                *received_b_clone.lock().unwrap() = Some(msg);
            })
            .unwrap();

        broadcast.send(42).unwrap();

        assert_eq!(*received_a.lock().unwrap(), Some(42));
        assert_eq!(*received_b.lock().unwrap(), Some(42));
    }

    #[test]
    fn should_allow_unsubscribe() {
        let broadcast = Broadcast::new();
        let received = Arc::new(Mutex::new(None));

        let received_clone = Arc::clone(&received);
        let subscription = broadcast
            .subscribe(move |msg| {
                *received_clone.lock().unwrap() = Some(msg);
            })
            .unwrap();

        subscription.unsubscribe().unwrap();

        broadcast.send(42).unwrap();

        assert_eq!(*received.lock().unwrap(), None);
    }

    #[test]
    fn should_allow_resubscribe() {
        let broadcast = Broadcast::new();
        let received = Arc::new(Mutex::new(None));

        let mut subscription = broadcast.subscribe(|_| {}).unwrap();
        subscription.unsubscribe().unwrap();

        let received_clone = Arc::clone(&received);
        subscription
            .resubscribe(move |msg| {
                *received_clone.lock().unwrap() = Some(msg);
            })
            .unwrap();

        broadcast.send(42).unwrap();

        assert_eq!(*received.lock().unwrap(), Some(42));
    }

    #[test]
    fn should_handle_multiple_subscriptions() {
        let broadcast = Broadcast::new();
        let count = Arc::new(Mutex::new(0));

        let count_clone = Arc::clone(&count);
        let sub1 = broadcast
            .subscribe(move |_| {
                *count_clone.lock().unwrap() += 1;
            })
            .unwrap();

        let count_clone = Arc::clone(&count);
        let sub2 = broadcast
            .subscribe(move |_| {
                *count_clone.lock().unwrap() += 1;
            })
            .unwrap();

        broadcast.send(()).unwrap();

        assert_eq!(*count.lock().unwrap(), 2);

        sub1.unsubscribe().unwrap();
        broadcast.send(()).unwrap();

        assert_eq!(*count.lock().unwrap(), 3);

        sub2.unsubscribe().unwrap();
        broadcast.send(()).unwrap();

        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn should_handle_unsubscribe_error_when_weak_reference_dropped() {
        let subscription = {
            let broadcast = Broadcast::new();
            broadcast.subscribe(|_: i32| {}).unwrap()
        };

        let result = subscription.unsubscribe();
        assert!(matches!(result, Err(UnsubscribeError::NoConnected)));
    }

    #[test]
    fn should_handle_resubscribe_error_when_weak_reference_dropped() {
        let mut subscription = {
            let broadcast = Broadcast::new();
            broadcast.subscribe(|_: i32| {}).unwrap()
        };

        let result = subscription.resubscribe(|_: i32| {});
        assert!(result.is_err());
    }
}
