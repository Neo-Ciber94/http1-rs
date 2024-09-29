pub mod http_body;

use std::{
    convert::Infallible,
    fmt::Debug,
    fs::File,
    io::{BufReader, Chain, Cursor, Empty, Read, Take, Write},
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc,
    },
};

use crate::error::BoxError;
use http_body::HttpBody;

struct BoxBodyInner<B: HttpBody>(B);

impl<B: HttpBody> HttpBody for BoxBodyInner<B>
where
    B: HttpBody,
    B::Err: Into<BoxError>,
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

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct BoxBody(Box<dyn HttpBody<Err = BoxError, Data = Vec<u8>>>);

fn box_body<B>(body: B) -> BoxBody
where
    B: HttpBody + 'static,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    BoxBody(Box::new(BoxBodyInner(body)))
}

pub struct Body {
    inner: BoxBody,
}

impl Body {
    pub fn empty() -> Self {
        Self::new(())
    }

    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody + 'static,
        B::Err: Into<BoxError>,
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

    fn size_hint(&self) -> Option<usize> {
        self.inner.0.size_hint()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bytes<T>(Option<T>);

impl<T> Bytes<T> {
    pub fn new(data: T) -> Self {
        Bytes(Some(data))
    }
}

impl<T: AsRef<[u8]>> HttpBody for Bytes<T> {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        match self.0.take() {
            Some(data) => Ok(Some(data.as_ref().to_vec())),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.as_ref().map(|x| x.as_ref().len())
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body::new(Bytes::new(value))
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(value: &'a [u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl HttpBody for Empty {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(None)
    }
}

impl HttpBody for () {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(None)
    }
}

const BUFFER_SIZE: usize = 4096;

impl<R> HttpBody for BufReader<R>
where
    R: Read,
{
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<R> HttpBody for Box<R>
where
    R: Read,
{
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl HttpBody for File {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }

    fn size_hint(&self) -> Option<usize> {
        self.metadata().map(|m| m.len() as usize).ok()
    }
}

impl HttpBody for Arc<File> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }

    fn size_hint(&self) -> Option<usize> {
        self.metadata().map(|m| m.len() as usize).ok()
    }
}

impl<R: Read> HttpBody for Take<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<T: Read, U: Read> HttpBody for Chain<T, U> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<T: AsRef<[u8]>> HttpBody for Cursor<T> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl HttpBody for &mut Vec<u8> {
    type Err = ();
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.is_empty() {
            Ok(None)
        } else {
            let bytes = std::mem::take(*self);
            Ok(Some(bytes))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

fn read_next_bytes<R: Read>(read: &mut R) -> std::io::Result<Option<Vec<u8>>> {
    let mut buf = vec![0; BUFFER_SIZE];
    match Read::read(read, &mut buf) {
        Ok(0) => Ok(None),
        Ok(n) => Ok(Some(buf[0..n].to_vec())),
        Err(err) => Err(err),
    }
}

pub struct ChunkedBody<T>(Option<Receiver<T>>);

impl<T> ChunkedBody<T> {
    pub fn new() -> (Self, Sender<T>) {
        let (sender, recv) = channel();

        let this = ChunkedBody(Some(recv));
        (this, sender)
    }
}

impl<T> HttpBody for ChunkedBody<T>
where
    T: AsRef<[u8]> + 'static,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        fn send_chunk(chunk: &[u8]) -> Result<Vec<u8>, BoxError> {
            let size = chunk.len();
            let mut buf = Vec::with_capacity(size + 10); // 10 bytes for size in hex and CRLF

            write!(buf, "{size:X}\r\n")?;

            // Write the bytes
            buf.extend_from_slice(chunk);
            buf.extend_from_slice(b"\r\n");

            Ok(buf)
        }

        match self.0.as_mut() {
            Some(rx) => {
                // Try read the next chunk, if ready sends it
                match rx.try_recv() {
                    Ok(chunk) => return send_chunk(chunk.as_ref()).map(Some),
                    Err(TryRecvError::Disconnected) => {
                        let _ = self.0.take(); // Drop the receiver if the sender was disconnected
                        return Ok(Some(b"0\r\n\r\n".to_vec()));
                    }
                    Err(_) => {}
                }

                // Otherwise wait for the next chunk
                match rx.recv() {
                    Ok(chunk) => send_chunk(chunk.as_ref()).map(Some),
                    Err(err) => Err(err.into()),
                }
            }
            None => Ok(None),
        }
    }
}

impl<T: AsRef<[u8]> + 'static> From<ChunkedBody<T>> for Body {
    fn from(value: ChunkedBody<T>) -> Self {
        Body::new(value)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, Cursor, Empty, Write};
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::time::UNIX_EPOCH;

    use crate::body::http_body::HttpBody;
    use crate::body::{Body, Bytes};

    use super::ChunkedBody;

    fn read_all_body_data(body: &mut Body) -> Vec<u8> {
        let mut all_data = Vec::new();
        while let Ok(Some(chunk)) = body.read_next() {
            all_data.extend(chunk);
        }
        all_data
    }

    fn get_random_temp_file_path() -> PathBuf {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_name = format!("{timestamp}{id}");
        temp_dir.join(file_name)
    }

    #[test]
    fn should_read_bytes_body() {
        let bytes = Bytes::new(b"hello".to_vec());
        let mut body = Body::new(bytes);

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"hello");
    }

    #[test]
    fn should_read_empty_body() {
        let mut body = Body::new(Empty::default());

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_read_unit_body() {
        let mut body = Body::new(());

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_read_chunked_body() {
        let (stream, sender) = ChunkedBody::new();

        std::thread::spawn(move || {
            sender.send("Hello World!").unwrap();
        })
        .join()
        .unwrap();

        let mut body = Body::new(stream);
        assert_eq!(body.size_hint(), None);

        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"C\r\nHello World!\r\n0\r\n\r\n");
    }

    #[test]
    fn should_read_cursor_body() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data.clone());
        let mut body = Body::new(cursor);

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_bufreader_body() {
        let data = vec![1, 2, 3, 4, 5];
        let reader = BufReader::new(Cursor::new(data.clone()));
        let mut body = Body::new(reader);

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_file_body() {
        // Create a temporary file with some data
        let path = "temp_file.txt";
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Hello, world!").unwrap();
        }

        let file = File::open(path).unwrap();
        let mut body = Body::new(file);

        assert_eq!(body.size_hint(), Some(14));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!\n");

        // Clean up the temporary file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn should_read_arc_file_body() {
        // Create a temporary file with some data
        let file_path = get_random_temp_file_path();
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Hello, world!").unwrap();
        }

        let file = Arc::new(File::open(&file_path).unwrap());
        let mut body = Body::new(file);

        assert_eq!(body.size_hint(), Some(14));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!\n");

        // Clean up the temporary file
        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn should_read_vec_body() {
        let data = vec![1, 2, 3, 4, 5];
        let mut body = Body::from(data.clone());

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_string_body() {
        let mut body = Body::from("Hello".to_string());

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn should_read_str_body() {
        let mut body = Body::from("Hello, world!");

        assert_eq!(body.size_hint(), Some(13));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!");
    }
}
