use std::{
    fmt::Debug,
    io::{Read, Write},
    sync::{Arc, Mutex},
};

#[doc(hidden)]
pub trait Conn: Read + Write {}

impl<T> Conn for T where T: Read + Write {}

#[derive(Clone)]
pub struct Upgrade(Arc<Mutex<dyn Conn + Send + Sync>>);

impl Upgrade {
    pub(crate) fn new<C>(conn: C) -> Self
    where
        C: Conn + Sync + Send + 'static,
    {
        Upgrade(Arc::new(Mutex::new(conn)))
    }
}

impl Debug for Upgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Upgrade").finish()
    }
}

impl Read for Upgrade {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut lock = &mut *self.0.lock().unwrap();
        Read::read(&mut lock, buf)
    }
}

impl Write for Upgrade {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = &mut *self.0.lock().unwrap();
        Write::write(&mut lock, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut lock = &mut *self.0.lock().unwrap();
        Write::flush(&mut lock)
    }
}
