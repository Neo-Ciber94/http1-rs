use std::ops::Deref;

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
}

impl<T> Deref for NonEmptyList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            Inner::Single(value) => std::slice::from_ref(value),
            Inner::List(vec) => &vec,
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
