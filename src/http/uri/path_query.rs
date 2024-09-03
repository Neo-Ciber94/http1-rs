use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
