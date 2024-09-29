use std::{fmt::Display, str::FromStr};

use super::uri::InvalidUri;

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

        let port = parse_port(&port_str[1..])?;
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Authority;

    #[test]
    fn should_parse_full_authority() {
        let authority = Authority::from_str("user:pass@10.0.2.2:8080").unwrap();

        assert_eq!(authority.user_info(), Some("user:pass"));
        assert_eq!(authority.host(), "10.0.2.2");
        assert_eq!(authority.port(), Some(8080));
    }

    #[test]
    fn should_parse_host_name_and_port() {
        let authority = Authority::from_str("localhost:4321").unwrap();

        assert_eq!(authority.host(), "localhost");
        assert_eq!(authority.port(), Some(4321));
    }

    #[test]
    fn should_parse_ipv4_and_port() {
        let authority = Authority::from_str("127.0.0.1:9000").unwrap();

        assert_eq!(authority.host(), "127.0.0.1");
        assert_eq!(authority.port(), Some(9000));
    }

    #[test]
    fn should_parse_ipv6_and_port() {
        let authority = Authority::from_str("[2001:db8:85a3:0:0:8a2e:370:7334]:22300").unwrap();
        assert_eq!(authority.host(), "2001:db8:85a3:0:0:8a2e:370:7334");
        assert_eq!(authority.port(), Some(22300));
    }

    #[test]
    fn should_parse_only_host() {
        let authority = Authority::from_str("127.0.0.1").unwrap();

        assert_eq!(authority.host(), "127.0.0.1");
        assert_eq!(authority.port(), None);
    }

    #[test]
    fn should_fail_on_missing_port() {
        assert!(Authority::from_str("127.0.0.1:").is_err());
    }
}
