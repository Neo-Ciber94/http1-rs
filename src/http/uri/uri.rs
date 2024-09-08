use std::str::FromStr;

use super::{decode_uri_component, Authority, PathAndQuery, Scheme};

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
            write!(f, "{}", scheme)?;
        }

        if let Some(authority) = &self.authority {
            write!(f, "//{authority}")?;
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
    InvalidPort,
    EmptyUri,
}

impl FromStr for Uri {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = decode_uri_component(s).map_err(|_| InvalidUri::DecodeError)?;

        if s.trim().is_empty() {
            return Err(InvalidUri::EmptyUri);
        }

        // scheme
        let (scheme, rest) = parse_scheme(value)?;

        // authority
        let (authority, rest) = parse_authority(rest)?;

        // path and query
        let path_query = PathAndQuery::from_str(&rest)?;

        Ok(Uri::new(scheme, authority, path_query))
    }
}

fn parse_scheme(mut value: String) -> Result<(Option<Scheme>, String), InvalidUri> {
    if let Some(scheme_sep_idx) = value.find("://") {
        let scheme = Scheme::from(&value[..scheme_sep_idx]);
        let rest = value.split_off(scheme_sep_idx + 3);
        Ok((Some(scheme), rest))
    } else {
        Ok((None, value))
    }
}

fn parse_authority(mut value: String) -> Result<(Option<Authority>, String), InvalidUri> {
    if value.starts_with("/") {
        return Ok((None, value));
    }

    let mut user_info: Option<String> = None;
    let mut port: Option<u16> = None;
    let host: String;

    if let Some(user_info_sep_idx) = value.find("@") {
        user_info = value[..user_info_sep_idx].to_owned().into();
        value = value.split_off(user_info_sep_idx + 1);
    }

    // Ipv6 address
    if let Some(ipv6_start_idx) = value.find("[") {
        let ipv6_end_idx = value.find("]").ok_or(InvalidUri::InvalidHost)?;
        host = value[(ipv6_start_idx + 1)..ipv6_end_idx].to_owned().into();
        value = value.split_off(ipv6_end_idx + 1);
    }
    // Anything before the port separator is the host
    else if let Some(port_sep_idx) = value.find(":") {
        host = value[..port_sep_idx].to_owned();
        value = value.split_off(port_sep_idx); // the port
    }
    // If there is not port separator everything before the "/", "#" or "?" if the host
    else {
        let len = value.len();
        let slash_idx = value.find("/").unwrap_or(len);
        let fragment_idx = value.find("#").unwrap_or(len);
        let query_idx = value.find("?").unwrap_or(len);
        let sep_idx = slash_idx.min(fragment_idx).min(query_idx);
        host = value[..sep_idx].to_owned();
        value = value.split_off(sep_idx);
    }

    // Parse the port if any
    if let Some(port_sep_idx) = value.find(":") {
        // Parse the port
        let port_end_idx = value.find("/").unwrap_or(value.len());
        port = u16::from_str(&value[(port_sep_idx + 1)..port_end_idx])
            .map_err(|_| InvalidUri::InvalidPort)?
            .into();

        value = value.split_off(port_end_idx);
    }

    let authority = Authority::new(user_info, host, port);
    Ok((Some(authority), value))
}

#[cfg(test)]
mod tests {
    use crate::http::uri::{InvalidUri, Uri};
    use std::str::FromStr;

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

    // #[test]
    // fn should_fail_parse_only_with_path() {
    //     let uri_str = "this/is/a/path";
    //     assert_eq!(Uri::from_str(uri_str), Err(InvalidUri::InvalidHost))
    // }
}
