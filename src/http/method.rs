use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    CONNECT,
    TRACE,
    ExtensionMethod(String),
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::OPTIONS => "OPTIONS",
            Method::HEAD => "HEAD",
            Method::CONNECT => "CONNECT",
            Method::TRACE => "TRACE",
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
            v if v.eq_ignore_ascii_case("GET") => Method::GET,
            v if v.eq_ignore_ascii_case("POST") => Method::POST,
            v if v.eq_ignore_ascii_case("PUT") => Method::PUT,
            v if v.eq_ignore_ascii_case("DELETE") => Method::DELETE,
            v if v.eq_ignore_ascii_case("PATCH") => Method::PATCH,
            v if v.eq_ignore_ascii_case("OPTIONS") => Method::OPTIONS,
            v if v.eq_ignore_ascii_case("HEAD") => Method::HEAD,
            v if v.eq_ignore_ascii_case("CONNECT") => Method::CONNECT,
            v if v.eq_ignore_ascii_case("TRACE") => Method::TRACE,
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
            v if v.eq_ignore_ascii_case("GET") => Ok(Method::GET),
            v if v.eq_ignore_ascii_case("POST") => Ok(Method::POST),
            v if v.eq_ignore_ascii_case("PUT") => Ok(Method::PUT),
            v if v.eq_ignore_ascii_case("DELETE") => Ok(Method::DELETE),
            v if v.eq_ignore_ascii_case("PATCH") => Ok(Method::PATCH),
            v if v.eq_ignore_ascii_case("OPTIONS") => Ok(Method::OPTIONS),
            v if v.eq_ignore_ascii_case("HEAD") => Ok(Method::HEAD),
            v if v.eq_ignore_ascii_case("CONNECT") => Ok(Method::CONNECT),
            v if v.eq_ignore_ascii_case("TRACE") => Ok(Method::TRACE),
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
