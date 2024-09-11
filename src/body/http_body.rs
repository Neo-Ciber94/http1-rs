pub trait HttpBody {
    type Err;
    type Data: Into<Vec<u8>>;

    fn read(&mut self) -> Result<Option<Self::Data>, Self::Err>;

    fn size_hint(&self) -> Option<usize> {
        return None;
    }
}
