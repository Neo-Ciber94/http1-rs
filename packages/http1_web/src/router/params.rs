use http1::common::map::{IntoIter, Iter, OrderedMap};

pub type ParamsIter<'a> = Iter<'a, String, String>;

pub type ParamsIntoIter = IntoIter<String, String>;

/// The params for a route.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ParamsMap(pub(crate) OrderedMap<String, String>);

impl ParamsMap {
    /// Returns the value for the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|x| x.as_str())
    }

    /// Returns the param at the given position.
    pub fn get_index(&self, pos: usize) -> Option<&str> {
        self.0.get_index(pos).map(|x| x.as_str())
    }

    /// Returns `true` if the given key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns an iterator over the key-values.
    pub fn iter(&self) -> ParamsIter {
        self.0.iter()
    }

    /// Returns an iterator over the key-values.
    pub fn into_iter(self) -> ParamsIntoIter {
        self.0.into_iter()
    }
}
