use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

pub struct AnyMap(HashMap<TypeId, Box<dyn Any>>);

impl AnyMap {
    pub fn new() -> Self {
        AnyMap(Default::default())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|x| x.downcast_ref())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|x| x.downcast_mut())
    }

    pub fn contains<T>(&self) -> bool
    where
        T: 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.downcast().ok())
            .map(|x| *x)
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|x| x.downcast().ok())
            .map(|x| *x)
    }

    pub fn extend(&mut self, other: AnyMap) {
        self.0.extend(other.0);
    }
}

impl Default for AnyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for AnyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyMap").finish()
    }
}
