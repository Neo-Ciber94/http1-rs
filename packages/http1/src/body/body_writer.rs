use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use crate::error::BoxError;

use super::{http_body::HttpBody, Body};

pub struct BodyWriter<T>(Option<Receiver<T>>);

impl<T> BodyWriter<T> {
    pub fn new() -> (Self, Sender<T>) {
        let (sender, recv) = channel();

        let this = BodyWriter(Some(recv));
        (this, sender)
    }
}

impl<T> HttpBody for BodyWriter<T>
where
    T: AsRef<[u8]> + 'static,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        match self.0.as_mut() {
            Some(rx) => {
                // Try read the next chunk, if ready sends it
                match rx.try_recv() {
                    Ok(chunk) => return Ok(Some(chunk.as_ref().to_vec())),
                    Err(TryRecvError::Disconnected) => {
                        let _ = self.0.take(); // Drop the receiver if the sender was disconnected
                        return Ok(None);
                    }
                    Err(_) => {}
                }

                // Otherwise wait for the next chunk
                match rx.recv() {
                    Ok(chunk) => Ok(Some(chunk.as_ref().to_vec())),
                    Err(err) => Err(err.into()),
                }
            }
            None => Ok(None),
        }
    }
}

impl<T: AsRef<[u8]> + Send + 'static> From<BodyWriter<T>> for Body {
    fn from(value: BodyWriter<T>) -> Self {
        Body::new(value)
    }
}
