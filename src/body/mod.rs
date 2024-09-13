pub mod http_body;

use std::{
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display},
    io::{BufReader, Read, Write},
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use crate::error::BoxError;
use http_body::HttpBody;

struct BoxBodyInner<B: HttpBody>(B);

impl<B: HttpBody> HttpBody for BoxBodyInner<B>
where
    B: HttpBody,
    B::Err: Error + Send + Sync + 'static,
    B::Data: Into<Vec<u8>>,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.0
            .read_next()
            .map(|data| data.map(|x| x.into()))
            .map_err(|e| e.into())
    }
}

struct BoxBody(Box<dyn HttpBody<Err = BoxError, Data = Vec<u8>>>);

fn box_body<B>(body: B) -> BoxBody
where
    B: HttpBody + 'static,
    B::Err: Error + Send + Sync + 'static,
    B::Data: Into<Vec<u8>>,
{
    BoxBody(Box::new(BoxBodyInner(body)))
}

pub struct Body {
    inner: BoxBody,
}

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody + 'static,
        B::Err: Error + Send + Sync + 'static,
        B::Data: Into<Vec<u8>>,
    {
        let inner = box_body(body);
        Body { inner }
    }
}

impl HttpBody for Body {
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.inner.0.read_next()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

struct SizedBody(Option<Vec<u8>>);

impl HttpBody for SizedBody {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(self.0.take())
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.as_ref().map(|x| x.len())
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body::new(SizedBody(Some(value)))
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(value: &'a [u8]) -> Self {
        value.iter().cloned().collect::<Vec<_>>().into()
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.as_bytes().iter().cloned().collect::<Vec<_>>().into()
    }
}

impl<R> HttpBody for BufReader<R>
where
    R: Read,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        let mut buf = Vec::with_capacity(512);
        match Read::read(self, &mut buf) {
            Ok(0) => Ok(None),
            Ok(n) => Ok(Some(buf[0..n].to_vec())),
            Err(err) => Err(err.into()),
        }
    }
}

pub struct ChunkedBody(Option<Receiver<Vec<u8>>>);

impl ChunkedBody {
    pub fn new() -> (Self, Sender<Vec<u8>>) {
        let (sender, recv) = channel();

        let this = ChunkedBody(Some(recv));
        (this, sender)
    }
}

#[derive(Debug)]
pub struct ChunkedBodyError(BoxError);

impl Display for ChunkedBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ChunkedBodyError {}

impl HttpBody for ChunkedBody {
    type Err = ChunkedBodyError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        match self.0.as_mut() {
            Some(rx) => {
                // If disconnected, send the last chunk
                if let Err(TryRecvError::Disconnected) = rx.try_recv() {
                    let _ = self.0.take(); // Drop the receiver if the sender was disconnected
                    let mut buf = Vec::new();
                    write!(buf, "0\r\n\r\n").map_err(|e| ChunkedBodyError(e.into()))?;
                    return Ok(Some(buf));
                }

                match rx.recv() {
                    Ok(chunk) => {
                        let mut buf = Vec::new();
                        // Write the chunk size
                        let size = chunk.len();
                        write!(buf, "{size:X}\r\n").map_err(|e| ChunkedBodyError(e.into()))?;

                        // Write the chunk data
                        buf.write_all(&chunk)
                            .map_err(|e| ChunkedBodyError(e.into()))?;
                        write!(buf, "\r\n").map_err(|e| ChunkedBodyError(e.into()))?;

                        Ok(Some(buf))
                    }

                    Err(err) => Err(ChunkedBodyError(err.into())),
                }
            }
            None => Ok(None),
        }
    }
}

impl From<ChunkedBody> for Body {
    fn from(value: ChunkedBody) -> Self {
        Body::new(value)
    }
}
