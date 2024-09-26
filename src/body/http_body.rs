pub trait HttpBody {
    type Err;
    type Data: Into<Vec<u8>>;

    /**
     * Returns the next chunk of data.
     */
    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err>;

    /**
     * Returns the total size of the data to write.
     */
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
