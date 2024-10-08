use params::ParamsMap;
use route::Route;

pub mod params;
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

    /// Finds the route that matches the given path and get a reference to it.
    pub fn find(&'a self, path: &'a str) -> Option<Match<&'a T>> {
        self.0.find(path)
    }

    /// Finds the route that matches the given path and get a mutable reference to it.
    pub fn find_mut(&'a mut self, path: &'a str) -> Option<Match<&'a mut T>> {
        self.0.find_mut(path)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Route, &T)> {
        self.0.entries()
    }

    pub fn entries_mut(&'a mut self) -> impl Iterator<Item = (&Route, &mut T)> {
        self.0.entries_mut()
    }
}

impl<T> Default for Router<'_, T> {
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
