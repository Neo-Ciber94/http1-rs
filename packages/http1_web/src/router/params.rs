use std::collections::HashMap;

/// The params for a route.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ParamsMap(pub(crate) HashMap<String, String>);

impl ParamsMap {
    /// Returns the value for the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|x| x.as_str())
    }

    /// Returns `true` if the given key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns an iterator over the key-values.
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, String> {
        self.0.iter()
    }
}
