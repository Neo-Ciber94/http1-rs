use std::{fmt::Display, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    Http1_1,
}

impl Version {
    pub fn as_str(&self) -> &str {
        match self {
            Version::Http1_1 => "HTTP/1.1",
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct InvalidVersion {
    _priv: (),
}

impl FromStr for Version {
    type Err = InvalidVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("version? {s}: {}", s.eq_ignore_ascii_case("HTTP/1.1"));
        match s {
            s if s.eq_ignore_ascii_case("HTTP/1.1") => Ok(Version::Http1_1),
            _ => Err(InvalidVersion { _priv: () }),
        }
    }
}
