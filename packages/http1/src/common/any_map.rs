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
            self.0.insert(key, value.into_inner().into_any());
        }
    }
}

impl Debug for AnyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyMap").finish()
    }
}

#[doc(hidden)]
pub trait CloneBox: Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneBox + Send + Sync>;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

impl<T> CloneBox for T
where
    T: Any + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn CloneBox + Send + Sync> {
        Box::new(self.clone())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

struct CloneableBox(Box<dyn CloneBox + Send + Sync>);
impl CloneableBox {
    pub fn new<T>(value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        CloneableBox(Box::new(value))
    }

    pub fn into_inner(self) -> Box<dyn CloneBox + Send + Sync> {
        self.0
    }
}

impl Clone for CloneableBox {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

/// A map that holds data of any type that can be cloned.
#[derive(Default)]
pub struct CloneableAnyMap(HashMap<TypeId, CloneableBox>);

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
            .insert(TypeId::of::<T>(), CloneableBox::new(value))
            .and_then(|x| x.into_inner().into_any().downcast().ok())
            .map(|x| *x)
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn get<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .map(|x| x.0.clone_box())
            .and_then(|x| x.into_any().downcast().ok())
            .map(|x| *x)
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|x| x.into_inner().into_any().downcast().ok())
            .map(|x| *x)
    }

    pub fn extend(&mut self, other: CloneableAnyMap) {
        self.0.extend(other.0);
    }
}

impl Clone for CloneableAnyMap {
    fn clone(&self) -> Self {
        CloneableAnyMap(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::any_map::CloneableAnyMap;

    use super::AnyMap;

    #[test]
    fn test_any_map() {
        #[derive(Debug, PartialEq, Eq)]
        struct Sorcerer {
            name: &'static str,
        }

        let mut map = AnyMap::new();

        map.insert(Sorcerer {
            name: "Satoru Gojo",
        });
        map.insert(String::from("Infinite Void"));
        map.insert(2018_u32);
        map.insert((true, String::from("Nobara")));

        assert_eq!(map.len(), 4);
        assert!(map.contains::<Sorcerer>());
        assert!(!map.contains::<f32>());

        assert_eq!(
            map.get::<Sorcerer>().unwrap(),
            &Sorcerer {
                name: "Satoru Gojo"
            }
        );
        assert_eq!(map.get::<u32>().unwrap(), &2018);
        assert_eq!(map.get::<String>().unwrap(), "Infinite Void");
        assert_eq!(
            map.get::<(bool, String)>().unwrap(),
            &(true, String::from("Nobara"))
        );
        assert!(map.get::<Vec<u8>>().is_none());
    }

    #[test]
    fn test_cloneable_any_map() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Sorcerer {
            name: &'static str,
        }

        let mut map = CloneableAnyMap::new();

        map.insert(Sorcerer {
            name: "Megumi Fushiguro",
        });
        map.insert(String::from("Chimera Shadow Garden"));
        map.insert(2001_u32);

        assert_eq!(map.len(), 3);
        assert!(map.contains::<Sorcerer>());
        assert!(!map.contains::<f32>());

        assert_eq!(
            map.get::<Sorcerer>().unwrap(),
            Sorcerer {
                name: "Megumi Fushiguro"
            }
        );
        assert_eq!(map.get::<u32>().unwrap(), 2001_u32);
        assert_eq!(map.get::<String>().unwrap(), "Chimera Shadow Garden");
        assert!(map.get::<Vec<u8>>().is_none());
    }

    #[test]
    fn should_extend_any_map_from_cloneable_any_map() {
        let mut src = CloneableAnyMap::new();
        src.insert(String::from("Maki Zenin"));
        src.insert(90_040_i32);
        src.insert((true, false, false));

        let mut map = AnyMap::new();
        map.extend_from_cloneable(src);

        assert_eq!(map.len(), 3);
        assert_eq!(map.get::<String>().unwrap(), "Maki Zenin");
        assert_eq!(map.get::<i32>().unwrap(), &90_040_i32);
        assert_eq!(
            map.get::<(bool, bool, bool)>().unwrap(),
            &(true, false, false)
        );
    }
}
