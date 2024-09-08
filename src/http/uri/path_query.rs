use std::{collections::HashMap, fmt::Display, str::FromStr};

use super::InvalidUri;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathAndQuery {
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl PathAndQuery {
    pub fn new(path: String, query: Option<String>, fragment: Option<String>) -> Self {
        PathAndQuery {
            path,
            query,
            fragment,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    pub fn query_values(&self) -> QueryValues {
        match &self.query {
            Some(s) => QueryValues::Values { iter: s.split("&") },
            None => QueryValues::Empty,
        }
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

impl QueryValues<'_> {
    pub fn to_map(self) -> QueryMap {
        let mut map = HashMap::<String, QueryValue>::new();

        for (key, value) in self {
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
            QueryValue::List(list) => list.get(0).map(|s| s.as_str()),
        })
    }

    pub fn get_all(&self, key: impl AsRef<str>) -> GetAll {
        match self.0.get(key.as_ref()) {
            None => GetAll::Empty,
            Some(QueryValue::One(s)) => GetAll::Once(Some(s)),
            Some(QueryValue::List(list)) => GetAll::List {
                list: &list,
                pos: 0,
            },
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

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        println!("path_query: {s}");

        if s.is_empty() {
            return Ok(PathAndQuery::new("/".to_owned(), None, None));
        }

        // if !s.starts_with("/") {
        //     return Err(InvalidUri::InvalidPath);
        // }

        let mut fragment: Option<String> = None;
        let mut query: Option<String> = None;

        if let Some(fragment_idx) = s.find("#") {
            fragment = Some(s[(fragment_idx + 1)..].to_owned());
            s = &s[..fragment_idx];
        }

        if let Some(query_idx) = s.find("?") {
            query = Some(s[(query_idx + 1)..].to_owned());
            s = &s[..query_idx];
        }

        let path = if s.starts_with("/") {
            s.to_owned()
        } else {
            format!("/{s}")
        };

        Ok(PathAndQuery::new(path, query, fragment))
    }
}
