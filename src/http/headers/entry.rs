use super::{HeaderName, HeaderValue};

#[derive(Debug, Clone)]
pub enum EntryValue {
    Single(HeaderValue),
    List(Vec<HeaderValue>),
}

#[derive(Debug, Clone)]
pub struct HeaderEntry {
    pub(crate) key: HeaderName,
    pub(crate) value: EntryValue,
}

impl HeaderEntry {
    pub fn take(self) -> HeaderValue {
        match self.value {
            EntryValue::Single(x) => x,
            EntryValue::List(mut list) => list.remove(0),
        }
    }

    pub fn iter(&self) -> Iter {
        match &self.value {
            EntryValue::Single(x) => Iter::Once(Some(x)),
            EntryValue::List(values) => Iter::List { values, pos: 0 },
        }
    }

    pub fn into_iter(self) -> IntoIter {
        match self.value {
            EntryValue::Single(x) => IntoIter::Once(Some(x)),
            EntryValue::List(values) => IntoIter::List { values },
        }
    }
}

pub enum Iter<'a> {
    Once(Option<&'a HeaderValue>),
    List {
        values: &'a Vec<HeaderValue>,
        pos: usize,
    },
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Once(x) => x.take(),
            Iter::List { values, pos } => {
                let next = values.get(*pos)?;
                *pos += 1;
                Some(next)
            }
        }
    }
}

pub enum IntoIter {
    Once(Option<HeaderValue>),
    List { values: Vec<HeaderValue> },
}

impl Iterator for IntoIter {
    type Item = HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Once(x) => x.take(),
            IntoIter::List { values } => {
                if values.is_empty() {
                    None
                } else {
                    Some(values.remove(0))
                }
            }
        }
    }
}
