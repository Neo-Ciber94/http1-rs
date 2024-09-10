use std::{fmt::Display, str::FromStr};

use super::InvalidUri;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Authority {
    user_info: Option<String>,
    host: String,
    port: Option<u16>,
}

impl Authority {
    pub fn new(user_info: Option<String>, host: String, port: Option<u16>) -> Self {
        Authority {
            user_info,
            host,
            port,
        }
    }

    pub fn user_info(&self) -> Option<&str> {
        self.user_info.as_deref()
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user_info) = &self.user_info {
            write!(f, "{user_info}")?;
        }

        write!(f, "{}", self.host)?;

        if let Some(port) = &self.port {
            write!(f, ":{}", port)?;
        }

        Ok(())
    }
}

impl FromStr for Authority {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (user_info, rest) = match s.split_once("@") {
            None => (None, s),
            Some((user_info, rest)) => (Some(user_info.to_owned()), rest),
        };

        let (host, port) = parse_host_port(rest)?;

        Ok(Authority {
            user_info,
            host,
            port,
        })
    }
}

fn parse_host_port(s: &str) -> Result<(String, Option<u16>), InvalidUri> {
    fn parse_port(port_str: &str) -> Result<u16, InvalidUri> {
        u16::from_str(port_str).map_err(|_| InvalidUri::InvalidPort(port_str.to_owned()))
    }

    if s.starts_with("[") {
        let i = s.find("]").ok_or(InvalidUri::InvalidHost)?;
        let host = s[1..i].to_owned();
        let port_str = &s[(i + 1)..];

        if port_str.is_empty() {
            return Ok((host, None));
        }

        let port = parse_port(port_str)?;
        Ok((host, Some(port)))
    } else {
        match s.split_once(":") {
            Some((host_str, port_str)) => {
                let port = parse_port(port_str)?;
                Ok((host_str.to_owned(), Some(port)))
            }
            None => Ok((s.to_owned(), None)),
        }
    }
}
