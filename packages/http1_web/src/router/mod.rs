use std::collections::HashMap;

mod route;
mod simple_router;

/// A route matcher.
#[derive(Debug, Clone)]
pub struct Router<'a, T>(simple_router::SimpleRouter<'a, T>);

impl<'a, T> Router<'a, T> {
    /// Constructs a new router.
    pub fn new() -> Self {
        Router(simple_router::SimpleRouter::new())
    }

    /// Inserts a new route.
    ///
    /// # Route types
    /// - static: /home
    /// - dynamic: /users/:user_id
    /// - catch-all: /toys/:rest*
    pub fn insert(&mut self, route: &'a str, value: T) {
        self.0.insert(route, value);
    }

    /// Finds the route that matches the given path.
    pub fn find(&'a self, path: &'a str) -> Option<Match<'a, T>> {
        self.0.find(path)
    }
}

impl<T> Default for Router<'_, T> {
    fn default() -> Self {
        Router::new()
    }
}

/// The params for a route.
#[derive(Default, Clone, Debug)]
pub struct Params(pub(crate) HashMap<String, String>);

impl Params {
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

/// Represents a route match.
#[derive(Debug, Clone)]
pub struct Match<'a, T> {
    /// The params
    pub params: Params,

    /// The value of the match
    pub value: &'a T,
}
