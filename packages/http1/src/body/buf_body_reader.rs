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
        assert!(buffer_size > 0);

        let buf = vec![0; buffer_size].into_boxed_slice();

        BufBodyReader {
            reader: BufReader::new(reader),
            buf,
            eof: false,
        }
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
                println!("nothing was read");
                self.eof = true;
                Ok(None)
            }
            n => {
                println!("read: {n}");
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
