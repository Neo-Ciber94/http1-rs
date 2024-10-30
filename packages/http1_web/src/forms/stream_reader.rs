use std::io::{BufRead, BufReader, Read};

pub struct StreamReader<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
    total_bytes_read: usize,
}

impl<R: Read> StreamReader<R> {
    /// Constructs a new `StreamReader`.
    pub fn new(reader: R) -> Self {
        StreamReader {
            reader: BufReader::new(reader),
            buf: Vec::new(),
            total_bytes_read: 0,
        }
    }

    /// Returns the current number of bytes read.
    pub fn total_bytes_read(&self) -> usize {
        self.total_bytes_read
    }

    /// Read until the given bytes sequence, if the sequence is never found returns all the bytes.
    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> std::io::Result<(bool, &[u8])> {
        self.buf.clear();

        if sequence.is_empty() {
            return Ok((true, &[]));
        }

        let last_byte = sequence[sequence.len() - 1];
        let mut byte_buffer = Vec::new();
        let mut is_found = false;

        loop {
            let read_bytes = self.reader.read_until(last_byte, &mut byte_buffer)?;
            self.buf.extend_from_slice(&byte_buffer);
            self.total_bytes_read += read_bytes;

            if read_bytes == 0 {
                break;
            }

            if byte_buffer.ends_with(sequence) {
                is_found = true;
                break;
            }

            byte_buffer.clear();
        }

        Ok((is_found, &self.buf))
    }

    /// Read exactly the given number of bytes, returns an error if is unable to read the exact number of bytes.
    pub fn read_exact(&mut self, exact_bytes_count: usize) -> std::io::Result<&[u8]> {
        self.buf.clear();

        if exact_bytes_count == 0 {
            return Ok(&[]);
        }

        self.buf.reserve(exact_bytes_count);
        self.buf
            .extend(std::iter::repeat(0).take(exact_bytes_count));

        let fixed_size_buffer = &mut self.buf[..exact_bytes_count];

        let bytes_read = self.reader.read(fixed_size_buffer)?;
        self.total_bytes_read += bytes_read;

        if bytes_read != exact_bytes_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Failed to read exact",
            ));
        }

        Ok(&fixed_size_buffer[..bytes_read])
    }

    /// Read all the bytes until the end.
    pub fn read_to_end(&mut self) -> std::io::Result<&[u8]> {
        self.buf.clear();

        self.reader.read_to_end(&mut self.buf)?;
        self.total_bytes_read += self.buf.len();
        Ok(&self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::StreamReader;

    #[test]
    fn should_read_all() {
        let mut reader =
            StreamReader::new("Hello World! How are things doing over there?".as_bytes());
        let (_, first_sequence) = reader.read_until_sequence(b"World!").unwrap();

        assert_eq!(first_sequence, b"Hello World!");
        assert_eq!(reader.read_exact(1).unwrap(), b" ");
        assert_eq!(
            reader.read_to_end().unwrap(),
            b"How are things doing over there?"
        );

        assert_eq!(reader.total_bytes_read(), 45);
    }

    #[test]
    fn should_read_until_sequence() {
        let mut reader = StreamReader::new(b"Hello World! How are things?".as_ref());
        let (_, first_sequence) = reader.read_until_sequence(b"World!").unwrap();
        assert_eq!(first_sequence, b"Hello World!");
        assert_eq!(reader.total_bytes_read(), 12);
    }

    #[test]
    fn should_return_entire_input_if_sequence_not_found() {
        let mut reader: StreamReader<&[u8]> = StreamReader::new(b"Hello there!".as_ref());
        let (_, first_sequence) = reader.read_until_sequence(b"World!").unwrap();
        assert_eq!(first_sequence, b"Hello there!");
        assert_eq!(reader.total_bytes_read(), 12);
    }

    #[test]
    fn should_return_empty_when_empty_sequence_is_provided() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());
        let (_, result) = reader.read_until_sequence(b"").unwrap();

        assert_eq!(result, b"");
        assert_eq!(reader.total_bytes_read(), 0);
    }

    #[test]
    fn should_read_exact() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());
        let result = reader.read_exact(5).unwrap();

        assert_eq!(result, b"Hello");
        assert_eq!(reader.total_bytes_read(), 5);
    }

    #[test]
    fn should_return_empty_when_read_exact_zero_bytes() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());
        let result = reader.read_exact(0).unwrap();

        assert_eq!(result, b"");
        assert_eq!(reader.total_bytes_read(), 0);
    }

    #[test]
    fn should_error_when_read_exact_exceeds_available_data() {
        let mut reader = StreamReader::new(b"Hi".as_ref());
        let result = reader.read_exact(5);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn should_read_to_end() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());
        assert_eq!(reader.read_to_end().unwrap(), b"Hello World!");
        assert_eq!(reader.total_bytes_read(), 12);
    }

    #[test]
    fn should_return_empty_on_read_to_end_with_empty_input() {
        let mut reader = StreamReader::new(b"".as_ref());
        let result = reader.read_to_end().unwrap();

        assert_eq!(result, b"");
        assert_eq!(reader.total_bytes_read(), 0);
    }

    #[test]
    fn should_track_total_bytes_read_correctly_with_read_to_end() {
        let mut reader = StreamReader::new(b"Some sample text".as_ref());
        assert_eq!(reader.read_to_end().unwrap(), b"Some sample text");
        assert_eq!(reader.total_bytes_read(), 16);
    }
}
