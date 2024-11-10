use std::{borrow::Cow, fmt::Display, str::FromStr};

use super::{authority::Authority, path_query::PathAndQuery, scheme::Scheme};

// https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#Syntax
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uri {
    scheme: Option<Scheme>,
    authority: Option<Authority>,
    path_query: PathAndQuery,
}

impl Uri {
    pub fn new(
        scheme: Option<Scheme>,
        authority: Option<Authority>,
        path_query: PathAndQuery,
    ) -> Self {
        Uri {
            scheme,
            authority,
            path_query,
        }
    }

    pub fn scheme(&self) -> Option<&Scheme> {
        self.scheme.as_ref()
    }

    pub fn authority(&self) -> Option<&Authority> {
        self.authority.as_ref()
    }

    pub fn path_and_query(&self) -> &PathAndQuery {
        &self.path_query
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scheme) = &self.scheme {
            write!(f, "{}//", scheme)?;
        }

        if let Some(authority) = &self.authority {
            write!(f, "{authority}")?;
        }

        write!(f, "{}", self.path_query)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidUri {
    DecodeError,
    InvalidScheme,
    InvalidHost,
    InvalidPath,
    InvalidQuery,
    EmptyHost,
    InvalidPort(String),
    EmptyUri,
}

impl Display for InvalidUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidUri::DecodeError => write!(f, "Failed to decode URI"),
            InvalidUri::InvalidScheme => write!(f, "Invalid URI scheme"),
            InvalidUri::InvalidHost => write!(f, "Invalid URI host"),
            InvalidUri::InvalidPath => write!(f, "Invalid URI path"),
            InvalidUri::InvalidQuery => write!(f, "Invalid URI query"),
            InvalidUri::EmptyHost => write!(f, "Empty host in URI"),
            InvalidUri::InvalidPort(port) => write!(f, "Invalid port in URI: {}", port),
            InvalidUri::EmptyUri => write!(f, "Empty URI"),
        }
    }
}

impl FromStr for Uri {
    type Err = InvalidUri;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err(InvalidUri::EmptyUri);
        }

        // scheme
        let scheme: Option<Scheme> = s
            .find("://")
            .map(|scheme_idx| Scheme::from(&s[..scheme_idx]));

        if let Some(scheme) = &scheme {
            s = &s[(scheme.as_str().len() + 3)..];
        }

        // authority
        let mut authority: Option<Authority> = None;
        let mut path_start = 0;

        let authority_str = {
            if s.starts_with("/") {
                None
            } else {
                for (i, c) in s.as_bytes().iter().enumerate() {
                    match c {
                        b'#' | b'?' | b'/' => {
                            path_start = i;
                            break;
                        }
                        _ => {}
                    }
                }

                if path_start > 0 {
                    Some(&s[..path_start])
                } else {
                    Some(s)
                }
            }
        };

        if let Some(s) = authority_str {
            authority = Some(Authority::from_str(s)?);
        }

        // path and query
        let path_query_str = {
            let pq = match authority_str {
                Some(a) => &s[a.len()..],
                None => s,
            };

            if pq.starts_with("/") {
                Cow::from(pq)
            } else {
                Cow::from(format!("/{pq}"))
            }
        };

        let path_query = PathAndQuery::from_str(&path_query_str)?;

        Ok(Uri::new(scheme, authority, path_query))
    }
}

impl TryFrom<String> for Uri {
    type Error = InvalidUri;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Uri::from_str(value.as_str())
    }
}

impl<'a> TryFrom<&'a str> for Uri {
    type Error = InvalidUri;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Uri::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::uri::uri::Uri;

    #[test]
    fn should_parse_uri_with_empty_path() {
        let uri_str = "http://www.rust-lang.org";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "http");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.rust-lang.org");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/");
        assert_eq!(uri.path_and_query().query(), None);
    }

    #[test]
    fn should_parse_uri_without_path() {
        let uri_str = "https://www.rust-lang.org/";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.rust-lang.org");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/");
        assert_eq!(uri.path_and_query().query(), None);
    }

    #[test]
    fn should_parse_uri_with_full_components() {
        let uri_str = "https://john.doe@www.example.com:1234/forum/questions/?tag=networking&order=newest#top";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), Some("john.doe"));
        assert_eq!(authority.host(), "www.example.com");
        assert_eq!(authority.port(), Some(1234));
        assert_eq!(uri.path_and_query().path(), "/forum/questions/");
        assert_eq!(
            uri.path_and_query().query(),
            Some("tag=networking&order=newest")
        );
        assert_eq!(uri.path_and_query().fragment(), Some("top"));
    }

    #[test]
    fn should_parse_uri_without_scheme() {
        let uri_str = "localhost:5000/api/posts?limit=100&sort=asc";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme(), None);
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "localhost");
        assert_eq!(authority.port(), Some(5000));
        assert_eq!(uri.path_and_query().path(), "/api/posts");
        assert_eq!(uri.path_and_query().query(), Some("limit=100&sort=asc"));
    }

    #[test]
    fn should_parse_uri_with_fragment() {
        let uri_str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ#never";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.youtube.com");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/watch");
        assert_eq!(uri.path_and_query().query(), Some("v=dQw4w9WgXcQ"));
        assert_eq!(uri.path_and_query().fragment(), Some("never"));
    }

    #[test]
    fn should_parse_uri_only_with_fragment() {
        let uri_str = "https://www.example.com#hello-world";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.example.com");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/");
        assert_eq!(uri.path_and_query().query(), None);
        assert_eq!(uri.path_and_query().fragment(), Some("hello-world"));
    }

    #[test]
    fn should_parse_uri_only_with_query() {
        let uri_str = "https://www.example.com?name=value";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.example.com");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/");
        assert_eq!(uri.path_and_query().query(), Some("name=value"));
        assert_eq!(uri.path_and_query().fragment(), None);
    }

    #[test]
    fn should_parse_fragment_with_query() {
        let uri_str = "https://www.example.com#fragment?name=value";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "www.example.com");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/");
        assert_eq!(uri.path_and_query().query(), None);
        assert_eq!(uri.path_and_query().fragment(), Some("fragment?name=value"));
    }

    #[test]
    fn should_parse_ipv6_uri() {
        let uri_str = "ldap://[2001:db8::7]/c=GB?objectClass?one";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "ldap");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();

        assert_eq!(authority.user_info(), None);
        assert_eq!(authority.host(), "2001:db8::7");
        assert_eq!(authority.port(), None);
        assert_eq!(uri.path_and_query().path(), "/c=GB");
        assert_eq!(uri.path_and_query().query(), Some("objectClass?one"));
        assert_eq!(uri.path_and_query().fragment(), None);
    }

    #[test]
    fn should_parse_only_with_path() {
        let uri_str = "/api/senpai/is/otonoko?name=Makoto";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme(), None);
        assert_eq!(uri.authority(), None);

        assert_eq!(uri.path_and_query().path(), "/api/senpai/is/otonoko");
        assert_eq!(uri.path_and_query().query(), Some("name=Makoto"));
        assert_eq!(uri.path_and_query().fragment(), None);
    }
}
