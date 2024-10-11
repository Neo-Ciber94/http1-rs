use std::io::Read;

use super::http_body::HttpBody;

const BUFFER_SIZE: usize = 4096;

pub struct BodyReader<R> {
    reader: R,
    buffer: Vec<u8>,
    read_bytes: usize,
    expected_bytes: Option<usize>,
}

impl BodyReader<()> {
    pub fn new<R>(reader: R, expected_bytes: Option<usize>) -> BodyReader<R>
    where
        R: Read + 'static,
    {
        BodyReader {
            reader,
            read_bytes: 0,
            buffer: vec![0; BUFFER_SIZE],
            expected_bytes,
        }
    }
}

impl<R: Read> HttpBody for BodyReader<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.buffer.clear();

        if let Some(expected_bytes) = self.expected_bytes {
            if self.read_bytes > expected_bytes {
                return Ok(None);
            }
        }

        match Read::read(&mut self.reader, &mut self.buffer) {
            Ok(0) => Ok(None),
            Ok(n) => {
                self.read_bytes += n;

                let size = self.expected_bytes.map(|x| x.min(n)).unwrap_or(n);
                let chunk = &self.buffer;
                Ok(Some(chunk[0..size].to_vec()))
            }
            Err(err) => Err(err),
        }
    }
}
