use std::{
    fs::File,
    io::{BufReader, Read},
};

use super::temp_file::TempFile;

const DEFAULT_THRESHOLD_BYTES: usize = 1024 * 1024 * 32; // 32mb

enum Reader<R> {
    Memory(BufReader<R>),
    File(BufReader<File>),
}

pub struct FileBufferingReader<R> {
    reader: Reader<R>,
    bytes_read: usize,
    threshold_bytes: usize,
    temp_handle: Option<TempFile>,
}

impl<R: Read> FileBufferingReader<R> {
    pub fn new(reader: R) -> Self {
        Self::with_threshold(reader, DEFAULT_THRESHOLD_BYTES)
    }

    pub fn with_threshold(reader: R, threshold_bytes: usize) -> Self {
        assert!(threshold_bytes > 0);

        FileBufferingReader {
            reader: Reader::Memory(BufReader::new(reader)),
            bytes_read: 0,
            threshold_bytes,
            temp_handle: None,
        }
    }

    fn switch_to_read_file(&mut self) -> std::io::Result<()> {
        todo!()
    }

    fn read_next(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Continue reading from memory
        if self.bytes_read + buf.len() > self.threshold_bytes {
            self.switch_to_read_file()?;
        }

        match &mut self.reader {
            Reader::Memory(reader) => reader.read(buf),
            Reader::File(reader) => reader.read(buf),
        }
    }
}
