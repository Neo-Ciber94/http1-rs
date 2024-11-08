use std::io::{BufReader, Read};

use super::form_data::PARSE_BUFFER_SIZE;

pub struct StreamReader<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
    inner_buf: Box<[u8]>,
    eof: bool,
    pos: usize,
    total_bytes_read: usize,
    reader_bytes_limit: usize,
}

#[allow(dead_code)]
impl<R: Read> StreamReader<R> {
    /// Constructs a new [`StreamReader`].
    pub fn new(reader: R) -> Self {
        Self::with_buffer_size(reader, PARSE_BUFFER_SIZE)
    }

    /// Constructs a new [`StreamReader`] with the given buffer size.
    pub fn with_buffer_size(reader: R, buffer_size: usize) -> Self {
        Self::with_buffer_size_and_read_limit(
            reader,
            buffer_size,
            http1::constants::DEFAULT_MAX_BODY_SIZE,
        )
    }

    /// Constructs a new [`StreamReader`] with the given max reader bytes limit.
    pub fn with_reader_bytes_limit(reader: R, reader_bytes_limit: usize) -> Self {
        Self::with_buffer_size_and_read_limit(reader, PARSE_BUFFER_SIZE, reader_bytes_limit)
    }

    /// Constructs a new [`StreamReader`] with the given buffer size and max reader bytes limit.
    pub fn with_buffer_size_and_read_limit(
        reader: R,
        buffer_size: usize,
        reader_bytes_limit: usize,
    ) -> Self {
        assert!(buffer_size > 0);
        assert!(reader_bytes_limit > 0);

        let inner_buf = vec![0; buffer_size].into_boxed_slice();

        StreamReader {
            reader: BufReader::new(reader),
            buf: Vec::new(),
            inner_buf,
            eof: false,
            pos: 0,
            total_bytes_read: 0,
            reader_bytes_limit,
        }
    }

    /// Whether if all the data was read.
    pub fn eof(&self) -> bool {
        self.eof
    }

    /// Current number of bytes read from the reader.
    pub fn total_bytes_read(&self) -> usize {
        self.total_bytes_read
    }

    /// Fill the buffer to ensure it contains at least `count` bytes.
    fn fill_buffer(&mut self, additional: usize) -> std::io::Result<usize> {
        if self.eof {
            return Ok(0);
        }

        let temp_buf = &mut self.inner_buf;
        let required = self.pos + additional;

        while self.buf.len() < required {
            let bytes_read = self.reader.read(temp_buf)?;

            if bytes_read == 0 {
                self.eof = true;
                break;
            }

            if self.total_bytes_read + bytes_read > self.reader_bytes_limit {
                let read = self.total_bytes_read + bytes_read;
                return Err(std::io::Error::other(format!(
                    "reader bytes limit reached: {read} > {}",
                    self.reader_bytes_limit
                )));
            }

            self.buf.extend_from_slice(&temp_buf[..bytes_read]);
            self.total_bytes_read += bytes_read;
            self.pos += bytes_read;
        }

        Ok(self.pos)
    }

    fn consume(&mut self, count: usize) -> Vec<u8> {
        let size = count.min(self.buf.len());
        let chunk = self.buf.drain(..size).collect::<Vec<_>>();
        self.pos -= size;

        // free memory
        self.buf.shrink_to_fit();

        chunk
    }

    /// Read until the specified byte with the given max bytes limit.
    ///
    /// If more bytes than the given are read an error is returned.
    pub fn read_until_with_limit(
        &mut self,
        byte: u8,
        max_bytes_limit: Option<usize>,
    ) -> std::io::Result<Vec<u8>> {
        fn check_limit(read: usize, limit: Option<usize>) -> std::io::Result<()> {
            let Some(limit) = limit else {
                return Ok(());
            };

            if read > limit {
                Err(std::io::Error::other(format!(
                    "limit of bytes reached: {read} > {limit}"
                )))
            } else {
                Ok(())
            }
        }

        let mut start_pos = 0;
        let mut read = 0;

        loop {
            if let Some(idx) = self.buf[start_pos..].iter().position(|&b| b == byte) {
                let offset = start_pos + idx + 1;
                check_limit(offset, max_bytes_limit)?;
                return Ok(self.consume(offset));
            }

            start_pos = self.buf.len();
            let bytes_read = self.fill_buffer(1)?;

            // We check the limit before incrementing the read count
            check_limit(read, max_bytes_limit)?;
            read += bytes_read;

            if bytes_read == 0 {
                break;
            }
        }

        Ok(self.consume(self.buf.len()))
    }

    /// Read until the specified byte.
    pub fn read_until(&mut self, byte: u8) -> std::io::Result<Vec<u8>> {
        self.read_until_with_limit(byte, None)
    }

    /// Peek the given number of bytes.
    pub fn peek(&mut self, count: usize) -> std::io::Result<&[u8]> {
        if self.buf.len() - self.pos < count {
            self.fill_buffer(count)?;
        }

        let len = count.min(self.buf.len());
        Ok(&self.buf[..len])
    }

    /// Read the next line with the given max bytes limit.
    pub fn read_line_with_limit(
        &mut self,
        max_bytes_limit: Option<usize>,
    ) -> std::io::Result<Vec<u8>> {
        self.read_until_with_limit(b'\n', max_bytes_limit)
    }

    /// Read the next line.
    pub fn read_line(&mut self) -> std::io::Result<Vec<u8>> {
        self.read_line_with_limit(None)
    }

    /// Read until the sequence is found.
    ///
    /// Because this method do not read all bytes at once but buffers them, this method should be called
    /// multiple times until the sequence is found or no more bytes are available to read.
    ///
    /// # Returns
    /// - `(false, bytes)` If the sequence was not found, and more bytes can be read.
    /// - `(false, empty)` If the sequence was no found and there is no more data to read.
    /// - `(true, bytes)` If the sequence is found
    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> std::io::Result<(bool, Vec<u8>)> {
        if sequence.is_empty() {
            return Ok((true, Vec::new()));
        }

        self.fill_buffer(sequence.len())?;

        if let Some(idx) = self
            .buf
            .windows(sequence.len())
            .position(|window| window == sequence)
        {
            let end = sequence.len() + idx;
            let result = self.consume(end);
            return Ok((true, result));
        }

        // Check for partial matches
        if !self.eof {
            if let Some((start_idx, _)) = overlapping_position(&self.buf, sequence) {
                self.fill_buffer(start_idx + sequence.len())?;

                let overlapping = &self.buf[start_idx..(start_idx + sequence.len())];

                if overlapping == sequence {
                    let result = self.consume(start_idx + sequence.len());
                    return Ok((true, result));
                }
            }
        }

        let rest = self.consume(self.buf.len());
        Ok((false, rest))
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

fn overlapping_position(slice: &[u8], sequence: &[u8]) -> Option<(usize, usize)> {
    assert!(!sequence.is_empty());

    if slice.len() == sequence.len() && slice == sequence {
        return Some((0, sequence.len()));
    }

    let first_byte = sequence[0];

    if let Some(idx) = slice.iter().position(|c| *c == first_byte) {
        let last = &slice[idx..];
        let remaining_len = last.len().min(sequence.len());
        let sequence_chunk = &sequence[..remaining_len];

        if sequence_chunk == last {
            return Some((idx, remaining_len));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::StreamReader;
    use std::io::Read;

    fn read_all_until_sequence<R: Read>(reader: &mut StreamReader<R>, sequence: &[u8]) -> Vec<u8> {
        let mut bytes = vec![];

        loop {
            let (found, chunk) = reader.read_until_sequence(sequence).unwrap();
            bytes.extend_from_slice(&chunk);

            if found || chunk.is_empty() {
                break;
            }
        }

        bytes
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
    fn should_return_entire_input_if_sequence_not_found() {
        let mut reader: StreamReader<&[u8]> = StreamReader::new(b"Hello there!".as_ref());
        let result = read_all_until_sequence(&mut reader, b"World");
        assert_eq!(result, b"Hello there!");
    }

    #[test]
    fn should_read_until_sequence_with_buffer_size() {
        let mut reader = StreamReader::with_buffer_size(
            "Hey how are you? Hey, I said how are you!?".as_bytes(),
            7,
        );

        let chunk1 = read_all_until_sequence(&mut reader, b"you");
        assert_eq!(chunk1, b"Hey how are you");

        let chunk2 = read_all_until_sequence(&mut reader, b"you");
        assert_eq!(chunk2, b"? Hey, I said how are you");
        assert_eq!(reader.read_exact(2).unwrap(), b"!?");
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
    fn should_read_line() {
        let mut reader = StreamReader::new(b"Hello World!\nThis is a test.\n".as_ref());

        // Reading the first line with Trim mode
        let line = reader.read_line().unwrap();
        assert_eq!(line, b"Hello World!\n");

        // Reading the second line with Trim mode
        let line = reader.read_line().unwrap();
        assert_eq!(line, b"This is a test.\n");
    }

    #[test]
    fn should_handle_empty_input_in_read_line() {
        let mut reader = StreamReader::new(b"".as_ref());

        // Trying to read from an empty input should return an empty slice
        let line = reader.read_line().unwrap();
        assert_eq!(line, b"");
        assert_eq!(reader.total_bytes_read(), 0);
    }

    #[test]
    fn should_handle_no_line_end_in_read_line() {
        let mut reader = StreamReader::new(b"Hello World!".as_ref());

        // Reading a line without a newline character
        let line = reader.read_line().unwrap();
        assert_eq!(line, b"Hello World!");
        assert_eq!(reader.total_bytes_read(), 12);
    }

    #[test]
    fn should_read_single_line_with_only_newline() {
        let mut reader = StreamReader::new(b"Line with newline\n".as_ref());

        // Reading with Trim mode should remove the newline
        let line = reader.read_line().unwrap();
        assert_eq!(line, b"Line with newline\n");
        assert_eq!(reader.total_bytes_read(), 18);
    }

    #[test]
    fn should_reach_reader_limit() {
        let data = std::iter::successors(Some(0), |n| Some((n + 1) % 255))
            .take(200)
            .collect::<Vec<u8>>();

        let mut reader = StreamReader::with_reader_bytes_limit(data.as_slice(), 100);
        let err = reader.read_to_end().unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::Other);
        assert!(err.to_string().contains("reader bytes limit reached"));
    }

    #[test]
    fn should_read_until_with_limit() {
        let data = "Time is an illusion that helps things makes sense, So we're always living in the present tense";

        let chunk1 = StreamReader::new(data.as_bytes()).read_until_with_limit(b'h', Some(10));
        assert!(chunk1.is_err());

        let chunk2 = StreamReader::new(data.as_bytes()).read_until_with_limit(b'h', Some(30));
        assert_eq!(chunk2.unwrap(), b"Time is an illusion th");
    }
}
