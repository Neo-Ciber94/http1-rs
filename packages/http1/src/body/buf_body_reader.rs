use std::io::{BufReader, Read};

use super::{http_body::HttpBody, Body};

const DEFAULT_BUFFER_SIZE: usize = 4 * 1024; // 4kb

pub struct BufBodyReader<R> {
    reader: BufReader<R>,
    buf: Box<[u8]>,
    eof: bool,
}

impl<R> BufBodyReader<R>
where
    R: Read + Send + 'static,
{
    pub fn new(reader: R) -> Self {
        Self::with_buffer_size(reader, DEFAULT_BUFFER_SIZE)
    }

    pub fn with_buffer_size(reader: R, buffer_size: usize) -> Self {
        Self::with_buf_reader_and_buffer_size(BufReader::new(reader), Some(buffer_size))
    }

    fn with_buf_reader_and_buffer_size(reader: BufReader<R>, buffer_size: Option<usize>) -> Self {
        let buffer_size = buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);

        assert!(buffer_size > 0);

        let buf = vec![0; buffer_size].into_boxed_slice();

        BufBodyReader {
            reader,
            buf,
            eof: false,
        }
    }
}

impl<R> From<BufReader<R>> for BufBodyReader<R>
where
    R: Read + Send + 'static,
{
    fn from(value: BufReader<R>) -> Self {
        Self::with_buf_reader_and_buffer_size(value, None)
    }
}

impl<R> HttpBody for BufBodyReader<R>
where
    R: Read + Send + 'static,
{
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.eof {
            return Ok(None);
        }

        let buf = &mut self.buf;

        match self.reader.read(buf)? {
            0 => {
                self.eof = true;
                Ok(None)
            }
            n => {
                let chunk = buf[..n].to_vec();
                Ok(Some(chunk))
            }
        }
    }
}

impl<R> From<BufBodyReader<R>> for Body
where
    R: Read + Send + 'static,
{
    fn from(value: BufBodyReader<R>) -> Self {
        Body::new(value)
    }
}
