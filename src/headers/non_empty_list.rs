use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum Inner<T> {
    Single(T),
    List(Vec<T>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NonEmptyList<T>(Inner<T>);

pub struct LastItemError;

impl<T> NonEmptyList<T> {
    pub fn single(value: T) -> Self {
        NonEmptyList(Inner::Single(value))
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Inner::Single(_) => 1,
            Inner::List(vec) => vec.len(),
        }
    }

    pub fn take_first(self) -> T {
        match self.0 {
            Inner::Single(x) => x,
            Inner::List(mut vec) => vec.remove(0),
        }
    }

    pub fn push(&mut self, value: T) {
        if let Inner::List(list) = &mut self.0 {
            list.push(value);
        } else {
            let Inner::Single(prev) = std::mem::replace(&mut self.0, Inner::List(vec![])) else {
                unreachable!()
            };

            self.0 = Inner::List(vec![prev, value])
        }
    }

    pub fn try_remove(&mut self, index: usize) -> Result<T, LastItemError> {
        if self.len() == 1 {
            return Err(LastItemError);
        }

        let Inner::List(list) = &mut self.0 else {
            unreachable!()
        };

        Ok(list.remove(index))
    }

    pub fn into_iter(self) -> IntoIter<T> {
        match self.0 {
            Inner::Single(x) => IntoIter(IteratorInner::Single(Some(x))),
            Inner::List(vec) => IntoIter(IteratorInner::Iter(vec.into_iter())),
        }
    }
}

impl<T> Deref for NonEmptyList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            Inner::Single(value) => std::slice::from_ref(value),
            Inner::List(vec) => vec,
        }
    }
}

impl<T> DerefMut for NonEmptyList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            Inner::Single(value) => std::slice::from_mut(value),
            Inner::List(vec) => vec.as_mut_slice(),
        }
    }
}

pub struct EmptyVecError;

impl<T> TryFrom<Vec<T>> for NonEmptyList<T> {
    type Error = EmptyVecError;

    fn try_from(mut value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EmptyVecError);
        }

        if value.len() == 1 {
            Ok(NonEmptyList::single(value.remove(0)))
        } else {
            Ok(NonEmptyList(Inner::List(value)))
        }
    }
}

pub type Iter<'a, T> = std::slice::Iter<'a, T>;

impl<'a, T> IntoIterator for &'a NonEmptyList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}

enum IteratorInner<T> {
    Single(Option<T>),
    Iter(std::vec::IntoIter<T>),
}

pub struct IntoIter<T>(IteratorInner<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IteratorInner::Single(x) => x.take(),
            IteratorInner::Iter(iter) => iter.next(),
        }
    }
}

impl<T> IntoIterator for NonEmptyList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Inner::Single(x) => IntoIter(IteratorInner::Single(Some(x))),
            Inner::List(vec) => IntoIter(IteratorInner::Iter(vec.into_iter())),
        }
    }
}
