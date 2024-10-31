use std::io::{BufReader, Read};

/// How to read a line
#[derive(Debug, PartialEq, Eq)]
pub enum ReadLineMode {
    /// Trim the `\n` or `\r\n` ending.
    Trim,

    /// Keep the line ending.
    Retain,
}

pub struct StreamReader<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
    eof: bool,
    pos: usize,
    total_bytes_read: usize,
}

impl<R: Read> StreamReader<R> {
    /// Constructs a new `StreamReader`.
    pub fn new(reader: R) -> Self {
        StreamReader {
            reader: BufReader::new(reader),
            buf: Vec::new(),
            eof: false,
            pos: 0,
            total_bytes_read: 0,
        }
    }

    /// Returns the current number of bytes read.
    pub fn total_bytes_read(&self) -> usize {
        self.total_bytes_read
    }

    /// Fill the buffer to ensure it contains at least `count` bytes.
    fn fill_buffer(&mut self, additional: usize) -> std::io::Result<usize> {
        if self.eof {
            return Ok(0);
        }

        let mut temp_buf = vec![0; 1024];

        while self.buf.len() < self.pos + additional {
            let bytes_read = self.reader.read(&mut temp_buf)?;

            if bytes_read == 0 {
                self.eof = true;
                break;
            }

            self.buf.extend_from_slice(&temp_buf[..bytes_read]);
            self.total_bytes_read += bytes_read;
            self.pos += bytes_read;
        }

        Ok(self.pos)
    }

    fn consume(&mut self, count: usize) -> Vec<u8> {
        let bytes = count.min(self.buf.len());
        let result = self.buf.drain(..bytes).collect::<Vec<_>>();
        self.pos -= bytes;
        result
    }

    /// Read until the specified byte.
    pub fn read_until(&mut self, byte: u8) -> std::io::Result<Vec<u8>> {
        let mut start_pos = 0;

        loop {
            if let Some(idx) = self.buf[start_pos..].iter().position(|&b| b == byte) {
                let offset = start_pos + idx + 1;
                return Ok(self.consume(offset));
            }

            start_pos = self.buf.len();
            let bytes_read = self.fill_buffer(1)?;

            if bytes_read == 0 {
                break;
            }
        }

        Ok(self.consume(self.buf.len()))
    }

    /// Peek the given number of bytes.
    pub fn peek(&mut self, count: usize) -> std::io::Result<&[u8]> {
        if self.buf.len() - self.pos < count {
            self.fill_buffer(count)?;
        }

        let len = count.min(self.buf.len());
        Ok(&self.buf[..len])
    }

    /// Read the next line.
    pub fn read_line(&mut self, mode: ReadLineMode) -> std::io::Result<Vec<u8>> {
        let mut line = self.read_until(b'\n')?;

        if mode == ReadLineMode::Trim {
            if line.ends_with(b"\r\n") {
                line.pop();
                line.pop();
            } else if line.ends_with(b"\n") {
                line.pop();
            }
        }

        Ok(line)
    }

    /// Read until the given bytes sequence, if the sequence is never found returns all the bytes.
    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> std::io::Result<(bool, Vec<u8>)> {
        if sequence.len() == 0 {
            return Ok((true, Vec::new()));
        }

        let sequence_len = sequence.len();
        let last_seq_byte = sequence[sequence.len() - 1];
        let mut start_pos = 0;

        loop {
            if let Some(idx) = self.buf[start_pos..]
                .iter()
                .rposition(|b| *b == last_seq_byte)
            {
                let slice = &self.buf[start_pos..=idx];

                dbg!(String::from_utf8_lossy(slice), start_pos, idx);
                if slice.ends_with(sequence) {
                    let chunk = self.consume(start_pos + idx + 1);
                    return Ok((true, chunk));
                }
            }

            let len = self.buf.len();
            let bytes_read = self.fill_buffer(sequence_len)?;
            start_pos = len;

            if bytes_read == 0 {
                break;
            }
        }

        Ok((false, std::mem::take(&mut self.buf)))
    }

    /// Read exactly the given number of bytes, returns a `(bool, Vec<u8>)`, the boolean determines whether if the exact number of bytes were read.
    pub fn read_exact(&mut self, exact_bytes_count: usize) -> std::io::Result<Vec<u8>> {
        self.fill_buffer(exact_bytes_count)?;
        let chunk = self.consume(exact_bytes_count);
        Ok(chunk)
    }

    /// Read all the bytes until the end.
    pub fn read_to_end(&mut self) -> std::io::Result<Vec<u8>> {
        while !self.eof {
            self.fill_buffer(1)?;
        }

        Ok(self.consume(self.buf.len()))
    }
}

#[cfg(test)]
mod tests {
    use crate::forms::stream_reader::ReadLineMode;

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
    fn should_read_until() {
        let mut reader = StreamReader::new("Adios amigos!".as_bytes());

        assert_eq!(reader.read_until(b' ').unwrap(), b"Adios ");
        assert_eq!(reader.read_until(b'!').unwrap(), b"amigos!");
    }

    #[test]
    fn should_peek_bytes_without_consuming() {
        let mut reader = StreamReader::new("Peek test example.".as_bytes());
        assert_eq!(reader.peek(5).unwrap(), b"Peek ");
        assert_eq!(reader.read_exact(5).unwrap(), b"Peek ");
    }

    #[test]
    fn should_peek_more_than_buffer_and_extend() {
        let mut reader = StreamReader::new("Short buffer peek test.".as_bytes());
        assert_eq!(reader.peek(0).unwrap(), b"");
        assert_eq!(reader.peek(1).unwrap(), b"S");
        assert_eq!(reader.peek(2).unwrap(), b"Sh");
        assert_eq!(reader.peek(3).unwrap(), b"Sho");
        assert_eq!(reader.peek(4).unwrap(), b"Shor");
        assert_eq!(reader.peek(5).unwrap(), b"Short");
        assert_eq!(reader.peek(6).unwrap(), b"Short ");
        assert_eq!(reader.peek(15).unwrap(), b"Short buffer pe");
        assert_eq!(reader.read_exact(15).unwrap(), b"Short buffer pe");
    }

    #[test]
    fn should_return_all_if_byte_not_found_in_read_until() {
        let mut reader = StreamReader::new("No exclamation here.".as_bytes());
        assert_eq!(reader.read_until(b'!').unwrap(), b"No exclamation here.");
    }

    #[test]
    fn should_read_until_repeated_byte() {
        let mut reader = StreamReader::new("aaa,bbb,ccc".as_bytes());
        assert_eq!(reader.read_until(b',').unwrap(), b"aaa,");
        assert_eq!(reader.read_until(b',').unwrap(), b"bbb,");
        assert_eq!(reader.read_until(b',').unwrap(), b"ccc");
    }

    #[test]
    fn should_read_until_sequence() {
        let mut reader = StreamReader::new(b"Hello World! How are things?".as_ref());
        let (_, first_sequence) = reader.read_until_sequence(b"World!").unwrap();

        dbg!(String::from_utf8_lossy(&first_sequence));
        assert_eq!(first_sequence, b"Hello World!");
    }

    #[test]
    fn should_return_entire_input_if_sequence_not_found() {
        let mut reader: StreamReader<&[u8]> = StreamReader::new(b"Hello there!".as_ref());
        let (_, first_sequence) = reader.read_until_sequence(b"World!").unwrap();
        assert_eq!(first_sequence, b"Hello there!");
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

        assert_eq!(result.len(), 5);
        assert_eq!(result, b"Hello");
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
        let result = reader.read_exact(5).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result, b"Hi");
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

    #[test]
    fn should_read_line_with_trim() {
        let mut reader = StreamReader::new(b"Hello World!\nThis is a test.\n".as_ref());

        // Reading the first line with Trim mode
        let line = reader.read_line(ReadLineMode::Trim).unwrap();
        assert_eq!(line, b"Hello World!");

        // Reading the second line with Trim mode
        let line = reader.read_line(ReadLineMode::Trim).unwrap();
        assert_eq!(line, b"This is a test.");
    }

    #[test]
    fn should_read_line_with_retain() {
        let mut reader = StreamReader::new(b"Hello World!\nThis is a test.\r\n".as_ref());

        // Reading the first line with Retain mode
        let line = reader.read_line(ReadLineMode::Retain).unwrap();
        assert_eq!(line, b"Hello World!\n");

        // Reading the second line with Retain mode
        let line = reader.read_line(ReadLineMode::Retain).unwrap();
        assert_eq!(line, b"This is a test.\r\n");
        assert_eq!(reader.total_bytes_read(), 30);
    }

    #[test]
    fn should_handle_empty_input_in_read_line() {
        let mut reader = StreamReader::new(b"".as_ref());

        // Trying to read from an empty input should return an empty slice
        let line = reader.read_line(ReadLineMode::Trim).unwrap();
        assert_eq!(line, b"");
        assert_eq!(reader.total_bytes_read(), 0);
    }

    #[test]
    fn should_handle_no_line_end_in_read_line() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());

        // Reading a line without a newline character
        let line = reader.read_line(ReadLineMode::Trim).unwrap();
        assert_eq!(line, b"Hello World!");
        assert_eq!(reader.total_bytes_read(), 12);
    }

    #[test]
    fn should_read_single_line_with_only_newline_trimmed() {
        let mut reader = StreamReader::new(b"Line with newline\n".as_ref());

        // Reading with Trim mode should remove the newline
        let line = reader.read_line(ReadLineMode::Trim).unwrap();
        assert_eq!(line, b"Line with newline");
        assert_eq!(reader.total_bytes_read(), 18);
    }

    #[test]
    fn should_read_single_line_with_only_newline_retained() {
        let mut reader = StreamReader::new(b"Line with newline\n".as_ref());

        // Reading with Retain mode should keep the newline
        let line = reader.read_line(ReadLineMode::Retain).unwrap();
        assert_eq!(line, b"Line with newline\n");
        assert_eq!(reader.total_bytes_read(), 18);
    }
}
