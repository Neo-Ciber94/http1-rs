pub trait HttpBody {
    type Err;
    type Data: Into<Vec<u8>>;

    /**
     * Returns the next chunk of data.
     */
    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err>;

    /**
     * After the not more chunks are available, return the last chunk to write if any.
     * This function will always be called once.
     */
    fn read_last(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        return Ok(None);
    }

    /**
     * Returns the total size of the data to write.
     */
    fn size_hint(&self) -> Option<usize> {
        return None;
    }
}
