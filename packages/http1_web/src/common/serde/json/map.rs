use std::collections::HashMap;
use std::hash::Hash;

/// A map that preserves the insertion order of keys.
#[derive(Debug, Clone)]
pub struct OrderedMap<K, V> {
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Creates a new, empty `OrderedMap`.
    pub fn new() -> Self {
        OrderedMap {
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }

    /// Inserts a key-value pair into the map.
    /// If the key is new, it is added to the `keys` vector to preserve order.
    /// If the key already exists, its value is updated.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
    }

    /// Retrieves a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Removes a key from the map, returning its value if it was present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.map.remove_entry(&key) {
            Some((k, v)) => {
                self.keys.retain(|x| x != &k);
                Some(v)
            }
            None => None,
        }
    }

    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Checks if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over the key-value pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(move |k| self.map.get(k).map(|v| (k, v)))
    }

    /// Returns an iterator over the key-value pairs in insertion order.
    pub fn into_iter(mut self) -> impl Iterator<Item = (K, V)> {
        self.keys.into_iter().filter_map(move |k| {
            let v = self.map.remove(&k).expect("missing value for key");
            Some((k, v))
        })
    }
}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            keys: Default::default(),
        }
    }
}
