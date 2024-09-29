use std::fmt::Display;

/// Any other scheme: ftp, mysql, etc...
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnyScheme(String); // this is just a private type to prevent create a raw scheme without using Scheme::from(...)

/// Represents an URI scheme.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    /// HTTP scheme.
    Http,
    /// HTTPS scheme.
    Https,
    /// Any other scheme: ftp, mysql, etc...
    Other(AnyScheme),
}

impl Scheme {
    /// Returns the str representation of this scheme.
    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Other(s) => s.0.as_str(),
        }
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
            Scheme::Other(s) => write!(f, "{}", s.0),
        }
    }
}

impl<'a> From<&'a str> for Scheme {
    fn from(value: &'a str) -> Self {
        match value {
            _ if value.eq_ignore_ascii_case("http") => Scheme::Http,
            _ if value.eq_ignore_ascii_case("https") => Scheme::Https,
            other => Scheme::Other(AnyScheme(other.to_lowercase().to_owned())),
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

#[cfg(test)]
mod tests {
    use crate::uri::scheme::AnyScheme;

    use super::Scheme;

    #[test]
    fn should_parse_http_scheme() {
        assert_eq!(Scheme::from("http"), Scheme::Http);
        assert_eq!(Scheme::from("HTTP"), Scheme::Http);
        assert_eq!(Scheme::from("hTTp"), Scheme::Http);
    }

    #[test]
    fn should_parse_https_scheme() {
        assert_eq!(Scheme::from("https"), Scheme::Https);
        assert_eq!(Scheme::from("HTTPS"), Scheme::Https);
        assert_eq!(Scheme::from("hTTpS"), Scheme::Https);
    }

    #[test]
    fn should_parse_any_scheme() {
        assert_eq!(
            Scheme::from("ftp"),
            Scheme::Other(AnyScheme("ftp".to_owned()))
        );
        assert_eq!(
            Scheme::from("POSTGRES"),
            Scheme::Other(AnyScheme("postgres".to_owned()))
        );
        assert_eq!(
            Scheme::from("Redis"),
            Scheme::Other(AnyScheme("redis".to_owned()))
        );
    }
}
