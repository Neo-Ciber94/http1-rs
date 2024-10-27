pub trait HttpBody {
    type Err;
    type Data: Into<Vec<u8>>;

    /// Returns the next chunk of data.
    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err>;

    /// Returns the total size of the data to write.
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Read all the chunks and returns a `Vec` containing all the bytes.
    fn read_all_bytes(&mut self) -> Result<Vec<u8>, Self::Err> {
        let mut bytes = Vec::new();

        while let Some(chunk) = self.read_next()? {
            bytes.extend(chunk.into());
        }

        Ok(bytes)
    }
}
