use super::HeaderName;

#[derive(Debug, Clone)]
pub enum EntryValue<T> {
    Single(T),
    List(Vec<T>),
}

#[derive(Debug, Clone)]
pub struct HeaderEntry<T> {
    pub(crate) key: HeaderName,
    pub(crate) value: EntryValue<T>,
}

impl<T> HeaderEntry<T> {
    pub fn take(self) -> T {
        match self.value {
            EntryValue::Single(x) => x,
            EntryValue::List(mut list) => list.remove(0),
        }
    }

    pub fn iter(&self) -> Iter<T> {
        match &self.value {
            EntryValue::Single(x) => Iter::Once(Some(x)),
            EntryValue::List(values) => Iter::List { values, pos: 0 },
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        match self.value {
            EntryValue::Single(x) => IntoIter::Once(Some(x)),
            EntryValue::List(values) => IntoIter::List { values },
        }
    }
}

pub enum Iter<'a, T> {
    Once(Option<&'a T>),
    List { values: &'a Vec<T>, pos: usize },
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

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

pub enum IntoIter<T> {
    Once(Option<T>),
    List { values: Vec<T> },
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

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
