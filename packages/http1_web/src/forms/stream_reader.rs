use std::io::{BufRead, BufReader, Read};

pub struct StreamReader<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
    total_bytes_read: usize,
}

impl<R: Read> StreamReader<R> {
    pub fn new(reader: R) -> Self {
        StreamReader {
            reader: BufReader::new(reader),
            buf: Vec::new(),
            total_bytes_read: 0,
        }
    }

    pub fn total_bytes_read(&self) -> usize {
        self.total_bytes_read
    }

    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> std::io::Result<&[u8]> {
        self.buf.clear();

        if sequence.is_empty() {
            return Ok(&[]);
        }

        let last_bytes = sequence[sequence.len() - 1];
        let mut byte_buffer = Vec::new();

        loop {
            let read_bytes = self.reader.read_until(last_bytes, &mut byte_buffer)?;
            self.buf.extend_from_slice(&byte_buffer);
            self.total_bytes_read += read_bytes;

            if read_bytes == 0 {
                break;
            }

            if byte_buffer.ends_with(sequence) {
                break;
            }

            byte_buffer.clear();
        }

        Ok(&self.buf)
    }

    pub fn read_exact(&mut self, bytes_to_read: usize) -> std::io::Result<&[u8]> {
        self.buf.clear();

        // We reverse in case the buffer is not big enough
        self.buf.reserve(bytes_to_read);
        let fixed_size_buffer = &mut self.buf[..bytes_to_read];

        // Read to fill the buffer
        let bytes_read = self.reader.read(fixed_size_buffer)?;
        self.total_bytes_read += bytes_read;

        Ok(&fixed_size_buffer[..bytes_read])
    }

    pub fn read_to_end(&mut self) -> std::io::Result<&[u8]> {
        self.buf.clear();

        self.reader.read_to_end(&mut self.buf)?;
        Ok(&self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::StreamReader;

    #[test]
    fn should_read_until_sequence() {
        let mut reader =
            StreamReader::new("Hello World! How are things doing over there?".as_bytes());
        let first_sequence = reader.read_until_sequence(b"World!").unwrap();

        assert_eq!(first_sequence, b"Hello World!");
        assert_eq!(reader.read_exact(1).unwrap(), b" ");
        assert_eq!(
            reader.read_to_end().unwrap(),
            b"How are things doing over there?"
        );
    }
}
