use params::ParamsMap;
use route::Route;

pub mod params;
mod route;
mod simple_router;

/// A route matcher.
#[derive(Debug, Clone)]
pub struct Router<T>(simple_router::SimpleRouter<T>);

impl<T> Router<T> {
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
    pub fn insert(&mut self, route: impl Into<String>, value: T) {
        self.0.insert(route, value);
    }

    /// Finds the route that matches the given path and get a reference to it.
    pub fn find(&self, path: &str) -> Option<Match<&T>> {
        self.0.find(path)
    }

    /// Finds the route that matches the given path and get a mutable reference to it.
    pub fn find_mut(&mut self, path: &str) -> Option<Match<&mut T>> {
        self.0.find_mut(path)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Route, &T)> {
        self.0.entries()
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = (&Route, &mut T)> {
        self.0.entries_mut()
    }

    pub fn into_entries(self) -> impl Iterator<Item = (Route, T)> {
        self.0.into_entries()
    }
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Router::new()
    }
}

/// Represents a route match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match<T> {
    /// The params
    pub params: ParamsMap,

    /// The value of the match
    pub value: T,
}
