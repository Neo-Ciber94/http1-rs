use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

pub(crate) trait Io: Read + Write {}

impl<T: Read + Write> Io for T {}

#[derive(Clone)]
pub struct IoStream(Arc<Mutex<dyn Io + Send + Sync>>);

impl IoStream {
    pub(crate) fn new<T>(io: T) -> Self
    where
        T: Read + Write + Send + Sync + 'static,
    {
        IoStream(Arc::new(Mutex::new(io)))
    }
}

impl Read for IoStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut lock = self.0.lock().expect("failed to get io lock");
        lock.read(buf)
    }
}

impl Write for IoStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = self.0.lock().expect("failed to get io lock");
        lock.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut lock = self.0.lock().expect("failed to get io lock");
        lock.flush()
    }
}
