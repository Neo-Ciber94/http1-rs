use std::{
    fmt::{Debug, Display},
    io::{Read, Write},
    sync::{Arc, Condvar, Mutex},
};

use super::connection::Connection;

#[derive(Debug)]
pub enum PendingUpgradeError {
    Failed,
    NoConnection,
}

impl std::error::Error for PendingUpgradeError {}

impl Display for PendingUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PendingUpgradeError::Failed => write!(f, "failed to upgrade connection"),
            PendingUpgradeError::NoConnection => write!(f, "no pending connection upgrade"),
        }
    }
}

#[derive(Default)]
enum UpgradeState {
    #[default]
    Waiting,
    Pending(Upgrade),
    Completed,
}

/// A pending connection upgrade.
#[derive(Clone)]
pub struct PendingUpgrade(Arc<(Mutex<UpgradeState>, Condvar)>);

/// A notifier that sends the upgraded connection when ready.
pub struct NotifyUpgradeReady(Arc<(Mutex<UpgradeState>, Condvar)>);

impl NotifyUpgradeReady {
    pub fn notify(self, upgrade: Upgrade) -> bool {
        let (mutex, cond_var) = &*self.0;
        match mutex.lock() {
            Ok(mut x) => {
                *x = UpgradeState::Pending(upgrade);
                cond_var.notify_one();
                true
            }
            Err(_) => false,
        }
    }
}

impl PendingUpgrade {
    pub(crate) fn new() -> (NotifyUpgradeReady, PendingUpgrade) {
        let pair = Arc::new((Mutex::new(UpgradeState::Waiting), Condvar::new()));
        (NotifyUpgradeReady(pair.clone()), PendingUpgrade(pair))
    }

    /// Wait for the connection upgrade to be available.
    pub fn wait(self) -> Result<Upgrade, PendingUpgradeError> {
        let (mutex, cond_var) = &*self.0;
        let mut lock = mutex.lock().map_err(|_| PendingUpgradeError::Failed)?;

        while matches!(*lock, UpgradeState::Waiting) {
            lock = cond_var
                .wait(lock)
                .map_err(|_| PendingUpgradeError::Failed)?;
        }

        match std::mem::take(&mut *lock) {
            UpgradeState::Pending(upgrade) => {
                let _ = std::mem::replace(&mut *lock, UpgradeState::Completed);
                Ok(upgrade)
            }
            UpgradeState::Completed => panic!("websocket upgrade was already completed"),
            UpgradeState::Waiting => unreachable!(),
        }
    }
}

impl Debug for PendingUpgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PendingUpgrade").finish()
    }
}

/// Provides the connection stream to write and read after a connection upgrade.
pub struct Upgrade(Connection);

impl Upgrade {
    pub(crate) fn new(conn: Connection) -> Self {
        Upgrade(conn)
    }

    pub fn try_clone(&self) -> Option<Self> {
        self.0.try_clone().map(Upgrade)
    }
}

impl Debug for Upgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Upgrade").finish()
    }
}

impl Read for Upgrade {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut self.0, buf)
    }
}

impl Write for Upgrade {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut self.0, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Write::flush(&mut self.0)
    }
}
