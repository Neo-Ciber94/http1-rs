use super::HeaderName;

#[derive(Debug, Clone)]
pub enum EntryValue {
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub(crate) key: HeaderName,
    pub(crate) value: EntryValue,
}

impl Entry {
    pub fn take(self) -> String {
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
    Once(Option<&'a str>),
    List { values: &'a Vec<String>, pos: usize },
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Once(x) => x.take(),
            Iter::List { values, pos } => {
                let next = values.get(*pos)?;
                *pos += 1;
                Some(next.as_str())
            }
        }
    }
}

pub enum IntoIter {
    Once(Option<String>),
    List { values: Vec<String> },
}

impl Iterator for IntoIter {
    type Item = String;

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
