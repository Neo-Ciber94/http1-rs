use std::{fmt::Display, str::FromStr};

/// Represents the http protocol version.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    // HTTP/1.1 version.
    #[default]
    Http1_1,
}

impl Version {
    /// Gets the version as `str`.
    pub fn as_str(&self) -> &str {
        match self {
            Version::Http1_1 => "HTTP/1.1",
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

///  An error when fails to parse the version.
#[derive(Debug)]
pub struct InvalidVersion {
    _priv: (),
}

impl std::error::Error for InvalidVersion {}

impl Display for InvalidVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid http version")
    }
}

impl FromStr for Version {
    type Err = InvalidVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case("HTTP/1.1") => Ok(Version::Http1_1),
            _ => Err(InvalidVersion { _priv: () }),
        }
    }
}
