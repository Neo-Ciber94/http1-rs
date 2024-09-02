use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    Connect,
    Trace,
    ExtensionMethod(String),
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Options => "OPTIONS",
            Method::Head => "HEAD",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::ExtensionMethod(ext) => ext.as_str(),
        }
    }
}

#[derive(Debug)]
pub struct InvalidMethod {
    _priv: (),
}

impl<'a> From<&'a str> for Method {
    fn from(value: &'a str) -> Self {
        match value {
            v if v.eq_ignore_ascii_case("GET") => Method::Get,
            v if v.eq_ignore_ascii_case("POST") => Method::Post,
            v if v.eq_ignore_ascii_case("PUT") => Method::Put,
            v if v.eq_ignore_ascii_case("DELETE") => Method::Delete,
            v if v.eq_ignore_ascii_case("PATCH") => Method::Patch,
            v if v.eq_ignore_ascii_case("OPTIONS") => Method::Options,
            v if v.eq_ignore_ascii_case("HEAD") => Method::Head,
            v if v.eq_ignore_ascii_case("CONNECT") => Method::Connect,
            v if v.eq_ignore_ascii_case("TRACE") => Method::Trace,
            _ => Method::ExtensionMethod(value.to_string()),
        }
    }
}

impl<'a> From<&'a String> for Method {
    fn from(value: &'a String) -> Self {
        Method::from(value.as_str())
    }
}

impl From<String> for Method {
    fn from(value: String) -> Self {
        Method::from(value.as_str())
    }
}

impl FromStr for Method {
    type Err = InvalidMethod;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            v if v.eq_ignore_ascii_case("GET") => Ok(Method::Get),
            v if v.eq_ignore_ascii_case("POST") => Ok(Method::Post),
            v if v.eq_ignore_ascii_case("PUT") => Ok(Method::Put),
            v if v.eq_ignore_ascii_case("DELETE") => Ok(Method::Delete),
            v if v.eq_ignore_ascii_case("PATCH") => Ok(Method::Patch),
            v if v.eq_ignore_ascii_case("OPTIONS") => Ok(Method::Options),
            v if v.eq_ignore_ascii_case("HEAD") => Ok(Method::Head),
            v if v.eq_ignore_ascii_case("CONNECT") => Ok(Method::Connect),
            v if v.eq_ignore_ascii_case("TRACE") => Ok(Method::Trace),
            _ => Err(InvalidMethod { _priv: () }),
        }
    }
}

// Implement the Display trait for the Method enum
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
