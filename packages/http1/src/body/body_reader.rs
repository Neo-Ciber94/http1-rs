use std::io::{BufRead, BufReader, Read};

use super::{http_body::HttpBody, Body};

const BUFFER_SIZE: usize = 4096;

pub struct FixedLengthBodyReader<R> {
    reader: R,
    buffer: Vec<u8>,
    read_bytes: usize,
    content_length: Option<usize>,
}

impl FixedLengthBodyReader<()> {
    pub fn new<R>(reader: R, content_length: Option<usize>) -> FixedLengthBodyReader<R>
    where
        R: Read,
    {
        FixedLengthBodyReader {
            reader,
            read_bytes: 0,
            buffer: vec![0; BUFFER_SIZE],
            content_length,
        }
    }
}

impl<R: Read> HttpBody for FixedLengthBodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.buffer.clear();

        if let Some(expected_bytes) = self.content_length {
            if self.read_bytes > expected_bytes {
                return Ok(None);
            }
        }

        match Read::read(&mut self.reader, &mut self.buffer) {
            Ok(0) => Ok(None),
            Ok(n) => {
                self.read_bytes += n;

                let size = self.content_length.map(|x| x.min(n)).unwrap_or(n);
                let chunk = &self.buffer;
                Ok(Some(chunk[0..size].to_vec()))
            }
            Err(err) => Err(err),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.content_length
    }
}

pub struct ChunkedBodyReader<R> {
    reader: BufReader<R>,
}

impl ChunkedBodyReader<()> {
    pub fn new<R>(reader: R) -> ChunkedBodyReader<R>
    where
        R: Read,
    {
        ChunkedBodyReader {
            reader: BufReader::new(reader),
        }
    }
}

impl<R: Read> HttpBody for ChunkedBodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        let mut str_buf = String::new();
        let mut byte_buf = Vec::new();

        // Read the chunk size line: {size in hex}\r\n
        self.reader.read_line(&mut str_buf)?;

        let chunk_length = usize::from_str_radix(str_buf.trim(), 16).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk length")
        })?;

        // End of chunks
        if chunk_length == 0 {
            return Ok(None);
        }

        // Read the chunk
        self.reader.read_exact(&mut byte_buf)?;

        Ok(Some(byte_buf))
    }
}

pub struct BodyReader {
    body: Body,
    chunk: Vec<u8>,
}

impl BodyReader {
    pub fn new(body: Body) -> Self {
        BodyReader {
            body,
            chunk: vec![],
        }
    }
}

impl Read for BodyReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut pos = 0;

        while pos < buf.len() {
            if self.chunk.is_empty() {
                let bytes = self.body.read_next().map_err(std::io::Error::other)?;
                match bytes {
                    Some(b) => self.chunk = b,
                    None => break,
                }
            }

            let left = buf.len() - pos;
            let len = left.min(self.chunk.len());

            for (idx, byte) in self.chunk.drain(..len).enumerate() {
                buf[pos + idx] = byte;
            }

            pos += len;
        }

        Ok(pos)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::body::Body;

    use super::BodyReader;

    #[test]
    fn should_read_complete_body() {
        let body = Body::from(b"Hello World".to_vec());
        let mut reader = BodyReader::new(body);

        let buf = &mut [0; 5];
        assert_eq!(reader.read(buf).unwrap(), 5);
        assert_eq!(buf, b"Hello");

        assert_eq!(reader.read(buf).unwrap(), 5);
        assert_eq!(buf, b" Worl");

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"dWorl");

        assert_eq!(reader.read(buf).unwrap(), 0);
    }

    #[test]
    fn should_return_0_for_empty_body() {
        let body = Body::from(Vec::new()); // Empty body
        let mut reader = BodyReader::new(body);

        let buf = &mut [0; 10];
        assert_eq!(reader.read(buf).unwrap(), 0); // No data to read
    }

    #[test]
    fn should_read_body_exactly_in_one_go() {
        let body = Body::from(b"Hello".to_vec());
        let mut reader = BodyReader::new(body);

        let buf = &mut [0; 5];
        assert_eq!(reader.read(buf).unwrap(), 5);
        assert_eq!(buf, b"Hello");

        assert_eq!(reader.read(buf).unwrap(), 0); // No more data to read
    }

    #[test]
    fn should_read_body_with_larger_buffer() {
        let body = Body::from(b"Hello".to_vec());
        let mut reader = BodyReader::new(body);

        let buf = &mut [0; 10]; // Buffer larger than body
        assert_eq!(reader.read(buf).unwrap(), 5); // Only 5 bytes available
        assert_eq!(buf[..5], b"Hello"[..]);

        assert_eq!(reader.read(buf).unwrap(), 0); // No more data to read
    }

    #[test]
    fn should_read_body_one_byte_at_a_time() {
        let body = Body::from(b"Hello".to_vec());
        let mut reader = BodyReader::new(body);

        let buf = &mut [0; 1]; // Single-byte buffer

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"H");

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"e");

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"l");

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"l");

        assert_eq!(reader.read(buf).unwrap(), 1);
        assert_eq!(buf, b"o");

        assert_eq!(reader.read(buf).unwrap(), 0); // No more data to read
    }
}
