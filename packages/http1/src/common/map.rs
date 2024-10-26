use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// A map that preserves the insertion order of keys.
#[derive(Clone)]
pub struct OrderedMap<K, V> {
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K, V> OrderedMap<K, V> {
    /// Creates a new, empty `OrderedMap`.
    pub fn new() -> Self {
        OrderedMap {
            map: HashMap::new(),
            keys: Vec::new(),
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

    /// Returns an iterator over the keys.
    pub fn keys(&self) -> std::slice::Iter<K> {
        self.keys.iter()
    }

    /// Remove all the entries in the map.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.map.clear();
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash,
{
    /// Retrieves a reference to the value corresponding to the key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
    {
        self.map.get(&key)
    }

    /// Retrieves a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
    {
        self.map.get_mut(&key)
    }

    /// Returns the reference to the element with the key at the given position.
    pub fn get_index(&self, index: usize) -> Option<&V> {
        match self.keys.get(index) {
            Some(key) => self.get(key),
            None => None,
        }
    }

    /// Returns the mutable reference to the element with the key at the given position.
    pub fn get_index_mut(&mut self, index: usize) -> Option<&mut V> {
        match self.keys.get(index) {
            Some(key) => self.map.get_mut(key),
            None => None,
        }
    }

    /// Returns `true` if the map contains the given key.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
    {
        self.map.contains_key(key)
    }

    /// Removes a key from the map, returning its value if it was present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
    {
        match self.map.remove_entry(key) {
            Some((k, v)) => {
                self.keys.retain(|x| x != &k);
                Some(v)
            }
            None => None,
        }
    }

    /// Removes the element at the given position.
    pub fn remove_index(&mut self, index: usize) -> Option<V> {
        let key = self.keys.get(index)?;
        match self.map.remove_entry(key) {
            Some((k, v)) => {
                self.keys.retain(|x| x != &k);
                Some(v)
            }
            None => None,
        }
    }

    /// Remove all elements that does not matches the predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        let mut i = 0;

        while i < self.keys.len() {
            let key = &self.keys[i];
            let value = self.map.get_mut(key).expect("expected value");
            let should_remove = !f(key, value);

            if should_remove {
                self.map.remove(key);
                self.keys.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Returns an iterator over the key-value pairs in insertion order.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            keys: self.keys.iter(),
            map: &self.map,
        }
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    /// Sort the keys of this map.
    pub fn sort_keys(&mut self) {
        self.keys.sort();
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Inserts a key-value pair into the map.
    /// If the key is new, it is added to the `keys` vector to preserve order.
    /// If the key already exists, its value is updated.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
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

impl<K, V> Debug for OrderedMap<K, V>
where
    K: Debug + Eq + Hash,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for k in self.keys() {
            let v = self.map.get(k).expect("value should exists");
            map.entry(k, v);
        }

        map.finish()
    }
}

pub struct Iter<'a, K, V> {
    keys: std::slice::Iter<'a, K>,
    map: &'a HashMap<K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Eq + Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().and_then(|k| {
            let v = self.map.get(k)?;
            Some((k, v))
        })
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter<K, V> {
    keys: std::vec::IntoIter<K>,
    map: HashMap<K, V>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Eq + Hash,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().and_then(|k| {
            let v = self.map.remove(&k)?;
            Some((k, v))
        })
    }
}

impl<K, V> IntoIterator for OrderedMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            keys: self.keys.into_iter(),
            map: self.map,
        }
    }
}

impl<K, V> PartialEq for OrderedMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.keys == other.keys
    }
}

impl<K, V> Eq for OrderedMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
}

#[cfg(test)]
mod tests {
    use super::OrderedMap;

    #[test]
    fn should_get_entries_in_order() {
        let mut map = OrderedMap::new();
        map.insert("first", 1);
        map.insert("second", 2);
        map.insert("third", 3);
        println!("{:?}", map);

        // Should get keys in order
        let keys = map.keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys, vec!["first", "second", "third"]);

        // Entries
        let mut iter = map.into_iter();
        assert_eq!(iter.next(), Some(("first", 1)));
        assert_eq!(iter.next(), Some(("second", 2)));
        assert_eq!(iter.next(), Some(("third", 3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_retain() {
        let mut map = OrderedMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        // Retain only the entries where the value is even
        map.retain(|_k, v| *v % 2 == 0);

        // After retain, only the entry with key "b" should remain
        let keys = map.keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys, vec!["b"]);

        // The value for "b" should be 2
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.get("a"), None);
        assert_eq!(map.get("c"), None);
    }

    #[test]
    fn test_sort_keys() {
        let mut map = OrderedMap::new();
        map.insert("banana", 1);
        map.insert("apple", 2);
        map.insert("cherry", 3);

        // Sort the keys
        map.sort_keys();

        // Keys should be in lexicographical order: ["apple", "banana", "cherry"]
        let keys = map.keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys, vec!["apple", "banana", "cherry"]);
    }
}
