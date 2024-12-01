use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

/// A map that holds data of any type.
#[derive(Default, Clone)]
pub struct AnyMap(HashMap<TypeId, Box<dyn CloneBox + Send + Sync>>);

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
            .and_then(|x| (**x).as_any().downcast_ref())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|x| (**x).as_any_mut().downcast_mut())
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: Send + Clone + Sync + 'static,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.into_any().downcast().ok())
            .map(|x| *x)
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|x| x.into_any().downcast().ok())
            .map(|x| *x)
    }

    pub fn extend(&mut self, other: AnyMap) {
        self.0.extend(other.0);
    }
}

impl Debug for AnyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyMap").finish()
    }
}

#[doc(hidden)]
pub trait CloneBox: Any {
    fn clone_box(&self) -> Box<dyn CloneBox + Send + Sync>;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for Box<dyn CloneBox + Send + Sync> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::AnyMap;

    #[test]
    fn test_any_map() {
        #[derive(Debug, Clone, PartialEq, Eq)]
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
}
