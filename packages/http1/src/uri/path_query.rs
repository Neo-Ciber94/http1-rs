use std::{
    collections::HashMap,
    fmt::Display,
    str::{FromStr, Split},
};

use super::uri::InvalidUri;

/// Represents the path and query from an URI.
///
/// `path_query = path ["?" query] ["#" fragment]`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathAndQuery {
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl PathAndQuery {
    /// Constructs a new `PathAndQuery`.
    pub fn new(path: String, query: Option<String>, fragment: Option<String>) -> Self {
        assert!(path.starts_with("/"), "Path should start with '/'");

        PathAndQuery {
            path,
            query,
            fragment,
        }
    }

    /// Constructs a new `PathAndQuery` from the given path.
    pub fn with_path(path: String) -> Self {
        Self::new(path, None, None)
    }

    /// Constructs a new `PathAndQuery` from the given path and query.
    pub fn with_path_query(path: String, query: String) -> Self {
        Self::new(path, Some(query), None)
    }

    /// Returns the path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the query if any or `None`.
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Returns the fragment if any or `None`.
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Returns an iterator over the query values.
    pub fn query_values(&self) -> QueryValues {
        match &self.query {
            Some(s) => QueryValues::Values { iter: s.split("&") },
            None => QueryValues::Empty,
        }
    }

    /// Create a map over the query values.
    pub fn query_map(&self) -> QueryMap {
        let mut map = HashMap::<String, QueryValue>::new();

        for (key, value) in self.query_values() {
            if map.contains_key(key) {
                let entry = map.get_mut(key).unwrap();
                match entry {
                    QueryValue::One(s) => {
                        let cur = std::mem::take(s);
                        let list = vec![cur, value.to_owned()];
                        *entry = QueryValue::List(list);
                    }
                    QueryValue::List(list) => list.push(value.to_owned()),
                }
            } else {
                map.insert(key.to_owned(), QueryValue::One(value.to_string()));
            }
        }

        QueryMap(map)
    }

    /// An iterator over the segments of the path.
    pub fn segments(&self) -> Segments {
        let mut p = self.path.as_str();

        if p.starts_with("/") {
            p = &p[1..];
        }

        if p.ends_with("/") {
            p = &p[..(p.len() - 1)];
        }

        Segments(p.split("/"))
    }
}

impl Display for PathAndQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;

        if let Some(query) = &self.query {
            write!(f, "?{query}")?;
        }

        if let Some(fragment) = &self.fragment {
            write!(f, "#{fragment}")?;
        }

        Ok(())
    }
}

pub struct Segments<'a>(Split<'a, &'a str>);

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub enum QueryValues<'a> {
    Empty,
    Values { iter: std::str::Split<'a, &'a str> },
}

impl<'a> Iterator for QueryValues<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            QueryValues::Empty => None,
            QueryValues::Values { iter } => {
                let raw = iter.next()?;
                let (name, value) = raw.split_once("=")?;
                Some((name, value))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryValue {
    One(String),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct QueryMap(HashMap<String, QueryValue>);

impl QueryMap {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.0.get(key.as_ref()).and_then(|s| match s {
            QueryValue::One(s) => Some(s.as_str()),
            QueryValue::List(list) => list.first().map(|s| s.as_str()),
        })
    }

    pub fn get_all(&self, key: impl AsRef<str>) -> GetAll {
        match self.0.get(key.as_ref()) {
            None => GetAll::Empty,
            Some(QueryValue::One(s)) => GetAll::Once(Some(s)),
            Some(QueryValue::List(list)) => GetAll::List { list, pos: 0 },
        }
    }

    pub fn contains(&self, key: impl AsRef<str>) -> bool {
        self.0.contains_key(key.as_ref())
    }
}

#[derive(Debug)]
pub enum GetAll<'a> {
    Empty,
    Once(Option<&'a String>),
    List { list: &'a [String], pos: usize },
}

impl<'a> Iterator for GetAll<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GetAll::Empty => None,
            GetAll::Once(s) => s.map(|x| x.as_str()).take(),
            GetAll::List { list, pos } => {
                let next = list.get(*pos)?;
                *pos += 1;
                Some(next.as_str())
            }
        }
    }
}

impl FromStr for PathAndQuery {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_path_query_string(s.to_owned())
    }
}

fn parse_path_query_string(mut s: String) -> Result<PathAndQuery, InvalidUri> {
    if s.is_empty() {
        return Ok(PathAndQuery::new("/".to_owned(), None, None));
    }

    if !s.starts_with("/") {
        return Err(InvalidUri::InvalidPath);
    }

    let mut fragment: Option<String> = None;
    let mut query: Option<String> = None;

    if let Some(fragment_idx) = s.find("#") {
        fragment = Some(s[(fragment_idx + 1)..].to_owned());
        let _ = s.split_off(fragment_idx);
    }

    if let Some(query_idx) = s.find("?") {
        query = Some(s[(query_idx + 1)..].to_owned());
        let _ = s.split_off(query_idx);
    }

    let path = if s.starts_with("/") {
        s.to_owned()
    } else {
        format!("/{s}")
    };

    Ok(PathAndQuery::new(path, query, fragment))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::PathAndQuery;

    #[test]
    fn should_parse_path() {
        let pq = PathAndQuery::from_str("/makoto/ryuji/saki").unwrap();

        assert_eq!(pq.path(), "/makoto/ryuji/saki");
        assert_eq!(pq.query(), None);
        assert_eq!(pq.fragment(), None);
    }

    #[test]
    fn should_parse_path_and_query() {
        let pq = PathAndQuery::from_str("/users/?limit=10&sort=email").unwrap();

        assert_eq!(pq.path(), "/users/");
        assert_eq!(pq.query(), Some("limit=10&sort=email"));
        assert_eq!(pq.fragment(), None);
    }

    #[test]
    fn should_parse_path_and_fragment() {
        let pq = PathAndQuery::from_str("/erwin#general").unwrap();

        assert_eq!(pq.path(), "/erwin");
        assert_eq!(pq.query(), None);
        assert_eq!(pq.fragment(), Some("general"));
    }

    #[test]
    fn should_get_query_map() {
        let pq = PathAndQuery::from_str("/users/?limit=10&sort=email").unwrap();

        let query_map = pq.query_map();
        assert_eq!(query_map.get("limit"), Some("10"));
        assert_eq!(query_map.get("sort"), Some("email"));
    }

    #[test]
    fn should_get_path_segments() {
        let p = PathAndQuery::from_str("/one/two/three").unwrap();
        let mut segments = p.segments();

        assert_eq!(segments.next(), Some("one"));
        assert_eq!(segments.next(), Some("two"));
        assert_eq!(segments.next(), Some("three"));
        assert_eq!(segments.next(), None);
    }

    #[test]
    fn should_get_path_segments_ending_in_slash() {
        let p = PathAndQuery::from_str("/one/two/three/").unwrap();
        let mut segments = p.segments();

        assert_eq!(segments.next(), Some("one"));
        assert_eq!(segments.next(), Some("two"));
        assert_eq!(segments.next(), Some("three"));
        assert_eq!(segments.next(), None);
    }

    #[test]
    fn should_get_path_segments_empty() {
        let p = PathAndQuery::from_str("/").unwrap();
        let mut segments = p.segments();

        assert_eq!(segments.next(), Some(""));
        assert_eq!(segments.next(), None);
    }

    #[test]
    fn should_get_single_segment() {
        let p = PathAndQuery::from_str("/one").unwrap();
        let mut segments = p.segments();

        assert_eq!(segments.next(), Some("one"));
        assert_eq!(segments.next(), None);
    }
}
