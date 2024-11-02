use std::io::{BufRead, BufReader, Read};

use super::{http_body::HttpBody, Body};

const DEFAULT_MAX_BODY_SIZE: usize = 64 * 1024; // 64mb
const DEFAULT_BUFFER_SIZE: usize = 4 * 1024; // 4kb

pub struct FixedLengthBodyReader<R> {
    reader: R,
    buffer: Box<[u8]>,
    read_bytes: usize,
    max_body_size: usize,
    content_length: Option<usize>,
}

impl FixedLengthBodyReader<()> {
    pub fn new<R>(
        reader: R,
        content_length: Option<usize>,
        max_body_size: Option<usize>,
    ) -> FixedLengthBodyReader<R>
    where
        R: Read,
    {
        let buffer = vec![0; DEFAULT_BUFFER_SIZE].into_boxed_slice();
        let max_body_size = max_body_size.unwrap_or(DEFAULT_MAX_BODY_SIZE);

        FixedLengthBodyReader {
            reader,
            read_bytes: 0,
            max_body_size,
            buffer,
            content_length,
        }
    }
}

impl<R: Read> HttpBody for FixedLengthBodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.read_bytes >= self.max_body_size {
            return Err(body_limit_error(self.read_bytes, self.max_body_size));
        }

        if let Some(content_length) = self.content_length {
            if self.read_bytes >= content_length {
                return Ok(None);
            }
        }

        let expected_len = match self.content_length {
            Some(content_length) => match content_length.checked_sub(self.read_bytes) {
                Some(n) => n.min(self.buffer.len()),
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid content length",
                    ));
                }
            },
            None => self.buffer.len(),
        };

        if expected_len == 0 {
            return Ok(None);
        }

        let buf = &mut self.buffer[..expected_len];

        match Read::read(&mut self.reader, buf)? {
            0 => {
                if self.content_length.is_none() {
                    Ok(None)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "body incomplete",
                    ))
                }
            }
            n => {
                let chunk = self.buffer[..n].to_vec();
                self.read_bytes += n;
                Ok(Some(chunk))
            }
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.content_length
    }
}

pub struct ChunkedBodyReader<R> {
    reader: BufReader<R>,
    read_bytes: usize,
    max_body_size: usize,
}

impl ChunkedBodyReader<()> {
    pub fn new<R>(reader: R, max_body_size: Option<usize>) -> ChunkedBodyReader<R>
    where
        R: Read,
    {
        let max_body_size = max_body_size.unwrap_or(DEFAULT_MAX_BODY_SIZE);

        ChunkedBodyReader {
            reader: BufReader::new(reader),
            read_bytes: 0,
            max_body_size,
        }
    }
}

impl<R: Read> HttpBody for ChunkedBodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.read_bytes >= self.max_body_size {
            return Err(body_limit_error(self.read_bytes, self.max_body_size));
        }

        let mut str_buf = String::new();
        let mut byte_buf = Vec::new();

        // Read the chunk size line: {size in hex}\r\n
        self.reader.read_line(&mut str_buf)?;
        self.read_bytes += str_buf.as_bytes().len();

        let chunk_length = usize::from_str_radix(str_buf.trim(), 16).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk length")
        })?;

        if self.read_bytes + chunk_length > self.max_body_size {
            return Err(body_limit_error(self.read_bytes, self.max_body_size));
        }

        // End of chunks
        if chunk_length == 0 {
            return Ok(None);
        }

        // Read the chunk
        self.reader.read_exact(&mut byte_buf)?;
        self.read_bytes += byte_buf.len();

        Ok(Some(byte_buf))
    }
}

fn body_limit_error(read_bytes: usize, max_body_size: usize) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Max request body size reached `{read_bytes} >= {max_body_size}` bytes",),
    )
}

pub struct BodyReader {
    body: Body,
    chunk: Vec<u8>,
    eof: bool,
}

impl BodyReader {
    pub fn new(body: Body) -> Self {
        BodyReader {
            body,
            chunk: vec![],
            eof: false,
        }
    }
}

impl BodyReader {
    fn read_chunk(&mut self) -> std::io::Result<()> {
        if self.eof {
            return Ok(());
        }

        match self.body.read_next().map_err(std::io::Error::other)? {
            Some(b) => self.chunk.extend(b),
            None => self.eof = true,
        }

        Ok(())
    }
}

impl Read for BodyReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        self.read_chunk()?;

        let mut read = 0;

        while !self.chunk.is_empty() && read < buf.len() {
            let chunk = &mut self.chunk;
            let remaining = buf.len() - read;
            let len = remaining.min(chunk.len());

            let src = &chunk[..len];
            let dst = &mut buf[read..(read + len)];
            dst.copy_from_slice(src);
            chunk.drain(..len);

            read += len;

            // Read the next chunk if no data is available
            if self.chunk.is_empty() {
                self.read_chunk()?;
            }
        }

        Ok(read)
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
