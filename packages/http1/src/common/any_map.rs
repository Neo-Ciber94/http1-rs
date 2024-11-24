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
pub trait CloneBox: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn DynClone + Send + Sync>;
    fn into_any_box(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

#[doc(hidden)]
pub trait DynClone: CloneBox {}

impl<T> CloneBox for T
where
    T: Any + DynClone + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn DynClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn into_any_box(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

impl DynClone for Box<dyn DynClone + '_> {}

impl Clone for Box<dyn DynClone + '_> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

/// A map that holds data of any type that can be cloned.
#[derive(Default)]
pub struct CloneableAnyMap(HashMap<TypeId, Box<dyn DynClone + Send + Sync>>);

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
        T: DynClone + Send + Sync + 'static,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.into_any_box().downcast().ok())
            .map(|x| *x)
    }

    pub fn contains<T>(&self) -> bool
    where
        T: DynClone + Send + Sync + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn get<T>(&self) -> Option<T>
    where
        T: DynClone + Send + Sync + 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .map(|x| x.clone_box())
            .and_then(|x| x.into_any_box().downcast().ok())
            .map(|x| *x)
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: DynClone + Send + Sync + 'static,
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

impl Clone for CloneableAnyMap {
    fn clone(&self) -> Self {
        let mut map = HashMap::with_capacity(self.len());
        for (k, v) in &self.0 {
            map.insert(*k, v.clone_box());
        }
        CloneableAnyMap(map)
    }
}

mod s {
    use std::{
        any::{Any, TypeId},
        collections::HashMap,
    };

    trait BoxClone {
        fn clone_box(&self) -> Box<dyn BoxClone + Send + Sync>;
        fn into_any(self: Box<Self>) -> Box<dyn Any>;
    }

    impl<T> BoxClone for T
    where
        T: Any + Send + Sync + Clone,
    {
        fn clone_box(&self) -> Box<dyn BoxClone + Send + Sync> {
            Box::new(self.clone())
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    struct CloneBox(Box<dyn BoxClone + Send + Sync>);
    impl CloneBox {
        pub fn new<T>(value: T) -> Self
        where
            T: Clone + Send + Sync + 'static,
        {
            CloneBox(Box::new(value))
        }
    }

    impl Clone for CloneBox {
        fn clone(&self) -> Self {
            CloneBox(self.0.clone_box())
        }
    }

    fn test() {
        let mut items: Vec<Box<dyn Any>> = vec![];

        let x = CloneBox::new(12);
        items.push(x.0.into_any());
    }

    struct CloneAnyMap(HashMap<TypeId, CloneBox>);
    impl CloneAnyMap {
        pub fn insert<T>(&mut self, value: T) -> Option<T>
        where
            T: Clone + Send + Sync + 'static,
        {
            self.0
                .insert(TypeId::of::<T>(), CloneBox::new(value))
                .map(|x| x.0.into_any())
                .and_then(|x| x.downcast().ok())
                .and_then(|x| *x)
        }

        pub fn get<T>(&self) -> Option<T>
        where
            T: Clone + Send + Sync + 'static,
        {
            self.0
                .get(&TypeId::of::<T>())
                .map(|x| x.0.clone_box())
                .map(|x| x.into_any())
                .and_then(|x| x.downcast().ok())
                .map(|x| *x)
        }
    }

    impl Clone for CloneAnyMap {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
}
