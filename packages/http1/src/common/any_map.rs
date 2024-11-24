use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

/// A map that holds data of any type.
#[derive(Default)]
pub struct AnyMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl AnyMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|x| x.downcast_ref())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|x| x.downcast_mut())
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.downcast().ok())
            .map(|x| *x)
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|x| x.downcast().ok())
            .map(|x| *x)
    }

    pub fn extend(&mut self, other: AnyMap) {
        self.0.extend(other.0);
    }

    pub fn extend_from_cloneable(&mut self, other: CloneableAnyMap) {
        for (key, value) in other.0 {
            self.0.insert(key, value.into_any_box());
        }
    }
}

impl Debug for AnyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyMap").finish()
    }
}

#[doc(hidden)]
pub trait CloneableAny: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneableAny + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_any_box(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

// impl dyn CloneableAny + Send + Sync {
//     pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
//         self.as_any().downcast_ref::<T>()
//     }

//     pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
//         self.as_any_mut().downcast_mut::<T>()
//     }
// }

impl<T> CloneableAny for T
where
    T: Any + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn CloneableAny + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any_box(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Default)]
pub struct CloneableAnyMap(HashMap<TypeId, Box<dyn CloneableAny + Send + Sync>>);

impl CloneableAnyMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.into_any_box().downcast().ok())
            .map(|x| *x)
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    // pub fn get<T>(&self) -> Option<T>
    // where
    //     T: Clone + Send + Sync + 'static,
    // {
    //     self.0
    //         .get(&TypeId::of::<T>())
    //         .map(|x| x.clone_box())
    //         .and_then(|x| x.into_any().downcast().ok())
    //         .map(|x| *x)
    // }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|x| x.into_any_box().downcast().ok())
            .map(|x| *x)
    }

    pub fn extend(&mut self, other: CloneableAnyMap) {
        self.0.extend(other.0);
    }
}

// impl Clone for CloneableAnyMap {
//     fn clone(&self) -> Self {
//         let mut map = HashMap::with_capacity(self.len());
//         for (k, v) in &self.0 {
//             map.insert(*k, v.clone_box());
//         }
//         CloneableAnyMap(map)
//     }
// }

