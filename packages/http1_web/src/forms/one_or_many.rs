use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub enum OneOrMany<T> {
    One(T),
    Many(Many<T>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Many<T>(Vec<T>);

impl<T> OneOrMany<T> {
    pub fn one(value: T) -> Self {
        OneOrMany::One(value)
    }

    pub fn many(values: Vec<T>) -> Self {
        assert!(values.len() > 0);
        OneOrMany::Many(Many(values))
    }

    pub fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 0,
            OneOrMany::Many(Many(vec)) => vec.len(),
        }
    }

    pub fn insert(&mut self, value: T) {
        if let OneOrMany::Many(Many(vec)) = self {
            vec.push(value);
        } else {
            let OneOrMany::One(existing) =
                std::mem::replace(self, OneOrMany::Many(Many(vec![value])))
            else {
                unreachable!()
            };

            let OneOrMany::Many(Many(vec)) = self else {
                unreachable!()
            };

            vec.insert(0, existing);
        }
    }

    pub fn first(&self) -> &T {
        match self {
            OneOrMany::One(x) => x,
            OneOrMany::Many(Many(vec)) => &vec[0],
        }
    }

    pub fn first_mut(&mut self) -> &mut T {
        match self {
            OneOrMany::One(x) => x,
            OneOrMany::Many(Many(vec)) => &mut vec[0],
        }
    }

    pub fn take_first(self) -> T {
        match self {
            OneOrMany::One(x) => x,
            OneOrMany::Many(Many(mut vec)) => vec.remove(0),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(x) => vec![x],
            OneOrMany::Many(Many(vec)) => vec,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        match self {
            OneOrMany::One(x) => Iter::One(Some(x)),
            OneOrMany::Many(Many(vec)) => Iter::Many(vec.iter()),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        match self {
            OneOrMany::One(x) => IterMut::One(Some(x)),
            OneOrMany::Many(Many(vec)) => IterMut::Many(vec.iter_mut()),
        }
    }
}

impl<T: Debug> Debug for OneOrMany<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OneOrMany::One(x) => write!(f, "{x:?}"),
            OneOrMany::Many(Many(vec)) => write!(f, "{vec:?}"),
        }
    }
}

pub enum Iter<'a, T> {
    One(Option<&'a T>),
    Many(std::slice::Iter<'a, T>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::One(x) => x.take(),
            Iter::Many(iter) => iter.next(),
        }
    }
}

pub enum IterMut<'a, T> {
    One(Option<&'a mut T>),
    Many(std::slice::IterMut<'a, T>),
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterMut::One(x) => x.take(),
            IterMut::Many(iter) => iter.next(),
        }
    }
}
