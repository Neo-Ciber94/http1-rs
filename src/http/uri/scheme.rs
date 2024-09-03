use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scheme {
    Http,
    Https,
    Other(String),
}

impl Scheme {
    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Other(s) => s.as_str(),
        }
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
            Scheme::Other(s) => write!(f, "{s}"),
        }
    }
}

impl<'a> From<&'a str> for Scheme {
    fn from(value: &'a str) -> Self {
        match value {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => Scheme::Other(other.to_owned()),
        }
    }
}

impl<'a> From<&'a String> for Scheme {
    fn from(value: &'a String) -> Self {
        Scheme::from(value.as_str())
    }
}

impl From<String> for Scheme {
    fn from(value: String) -> Self {
        Scheme::from(value.as_str())
    }
}
