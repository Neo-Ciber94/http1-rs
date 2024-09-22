use super::{non_empty_list::{self, NonEmptyList}, HeaderName, HeaderValue};

#[derive(Debug, Clone)]
pub struct HeaderEntry {
    pub(crate) key: HeaderName,
    pub(crate) value: NonEmptyList<HeaderValue>,
}

impl HeaderEntry {
    pub fn take(self) -> HeaderValue {
        self.value.take_first()
    }

    pub fn iter(&self) -> non_empty_list::Iter<HeaderValue> {
       self.value.iter()
    }

    pub fn into_iter(self) -> non_empty_list::IntoIter<HeaderValue> {
        self.value.into_iter()
    }
}