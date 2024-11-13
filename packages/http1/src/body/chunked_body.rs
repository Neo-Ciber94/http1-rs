use std::{
    io::{BufRead, BufReader, Read, Write},
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use crate::error::BoxError;

use super::{http_body::HttpBody, Body};

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

impl<T: AsRef<[u8]> + Send + 'static> From<ChunkedBody<T>> for Body {
    fn from(value: ChunkedBody<T>) -> Self {
        Body::new(value)
    }
}

pub struct ReadChunkedBody<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
    eof: bool,
}

impl<R> ReadChunkedBody<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        ReadChunkedBody {
            reader,
            buf: Vec::new(),
            eof: false,
        }
    }
}

impl<R> ReadChunkedBody<R>
where
    R: Read,
{
    fn read_line(&mut self, expected_len: usize) -> std::io::Result<Option<Vec<u8>>> {
        self.buf.reserve(expected_len + 2);

        let bytes_read = self.reader.read_until(b'\n', &mut self.buf)?;

        match bytes_read {
            0 => {
                self.eof = true;
                Ok(None)
            }
            _ => {
                if !self.buf.ends_with(b"\r\n") {
                    return Err(std::io::Error::other(
                        "Invalid chunk ending, expected `\\r\\n`",
                    ));
                }

                self.buf.pop();
                self.buf.pop();

                let data = std::mem::take(&mut self.buf);
                Ok(Some(data))
            }
        }
    }

    fn read_empty_line(&mut self) -> std::io::Result<()> {
        if let Some(line) = self.read_line(0)? {
            if !line.is_empty() {
                return Err(std::io::Error::other(format!(
                    "expected chunk ending `\r\n` but was `\r\n{:?}`",
                    String::from_utf8_lossy(&line)
                )));
            }
        }

        Ok(())
    }

    fn read_chunk_size(&mut self) -> std::io::Result<usize> {
        match self.read_line(1)? {
            Some(bytes) => {
                let hex = String::from_utf8_lossy(&bytes);
                let chunk_len = usize::from_str_radix(&hex, 16).map_err(std::io::Error::other)?;
                Ok(chunk_len)
            }
            None => Err(std::io::Error::other("chunk length not found")),
        }
    }

    fn read_chunk(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; size + 2].into_boxed_slice();
        self.reader.read_exact(&mut buf)?;

        if !buf.ends_with(b"\r\n") {
            return Err(std::io::Error::other(format!(
                "expected chunk ending `\r\n` after: `{:?}`",
                String::from_utf8_lossy(&buf)
            )));
        }

        let mut rest = buf.to_vec();
        rest.pop();
        rest.pop();

        Ok(rest)
    }
}

impl<R: Read> HttpBody for ReadChunkedBody<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.eof {
            return Ok(None);
        }

        let chunk_len = self.read_chunk_size()?;

        if chunk_len > 0 {
            let chunk = self.read_chunk(chunk_len)?;
            Ok(Some(chunk))
        } else {
            self.eof = true;
            self.read_empty_line()?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use crate::body::{body_reader::BodyReader, Body};

    use super::{ChunkedBody, ReadChunkedBody};

    #[track_caller]
    fn read_exact(mut reader: impl Read, size: usize) -> String {
        let mut buf = vec![0; size].into_boxed_slice();
        reader.read_exact(&mut buf).unwrap();
        String::from_utf8(buf.to_vec()).unwrap()
    }

    #[test]
    fn should_send_chunked_body() {
        let (body, sender) = ChunkedBody::new();

        std::thread::spawn(move || {
            for i in 1..=3 {
                sender.send(format!("Chunk {i}")).unwrap();
            }
        });

        let body = Body::new(body);

        let mut reader = BufReader::new(BodyReader::new(body));

        assert_eq!(read_exact(&mut reader, 3), "7\r\n");
        assert_eq!(read_exact(&mut reader, 9), "Chunk 1\r\n");

        assert_eq!(read_exact(&mut reader, 3), "7\r\n");
        assert_eq!(read_exact(&mut reader, 9), "Chunk 2\r\n");

        assert_eq!(read_exact(&mut reader, 3), "7\r\n");
        assert_eq!(read_exact(&mut reader, 9), "Chunk 3\r\n");

        assert_eq!(read_exact(&mut reader, 5), "0\r\n\r\n");
    }

    #[test]
    fn should_read_chunked_body() {
        let data = "7\r\nChunk 1\r\n7\r\nChunk 2\r\n7\r\nChunk 3\r\n0\r\n\r\n";
        let buf_reader = BufReader::new(data.as_bytes());

        let read_chunked_body = ReadChunkedBody::new(buf_reader);
        let body = Body::new(read_chunked_body);
        let mut reader = BodyReader::new(body);

        assert_eq!(read_exact(&mut reader, 7), "Chunk 1");
        assert_eq!(read_exact(&mut reader, 7), "Chunk 2");
        assert_eq!(read_exact(&mut reader, 7), "Chunk 3");
    }
}
