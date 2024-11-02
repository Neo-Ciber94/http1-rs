use std::io::{BufRead, BufReader, Read};

use super::{http_body::HttpBody, Body};

const BUFFER_SIZE: usize = 4 * 1024; // 4kb

pub struct FixedLengthBodyReader<R> {
    reader: R,
    buffer: Box<[u8]>,
    read_bytes: usize,
    content_length: Option<usize>,
}

impl FixedLengthBodyReader<()> {
    pub fn new<R>(reader: R, content_length: Option<usize>) -> FixedLengthBodyReader<R>
    where
        R: Read,
    {
        let buffer = vec![0; BUFFER_SIZE].into_boxed_slice();

        FixedLengthBodyReader {
            reader,
            read_bytes: 0,
            buffer,
            content_length,
        }
    }
}

impl<R: Read> HttpBody for FixedLengthBodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if let Some(content_length) = self.content_length {
            if self.read_bytes >= content_length {
                return Ok(None); // Updated to >= to prevent over-reading
            }
        }

        let expected_len = match self.content_length {
            Some(content_length) => match content_length.checked_sub(self.read_bytes) {
                Some(n) => n.min(self.buffer.len()),
                None => {
                    // The content length is less than the actual data length
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid content length",
                    ));
                }
            },
            None => {
                // Attempting to fill the buffer in absence of a known content length
                self.buffer.len()
            }
        };

        if expected_len == 0 {
            return Ok(None); // End of content reached
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

        let mut pos = 0;

        while !self.chunk.is_empty() && pos < buf.len() {
            let chunk = &mut self.chunk;
            let remaining = buf.len() - pos;
            let len = remaining.min(chunk.len());

            let src = &chunk[..len];
            let dst = &mut buf[pos..(pos + len)];
            dst.copy_from_slice(src);
            chunk.drain(..len);

            pos += len;

            // Read the next chunk if no data is available
            if self.chunk.is_empty() {
                self.read_chunk()?;
            }
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
