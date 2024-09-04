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

#[derive(Debug)]
pub enum InvalidUri {
    DecodeError,
    InvalidScheme,
    InvalidHost,
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
        let (scheme, rest) = parse_scheme(&value)?;

        // authority
        let (authority, rest) = parse_authority(rest)?;

        // path and query
        let path_query = parse_path_and_query(rest)?;

        Ok(Uri::new(scheme, authority, path_query))
    }
}

fn parse_scheme(value: &str) -> Result<(Option<Scheme>, &str), InvalidUri> {
    let (scheme_str, rest) = value.split_once(":").ok_or(InvalidUri::InvalidScheme)?;
    Ok((Some(Scheme::from(scheme_str)), rest))
}

fn parse_authority(mut value: &str) -> Result<(Option<Authority>, &str), InvalidUri> {
    if !value.starts_with("//") {
        return Ok((None, value));
    }

    // Remove the leading slash
    value = &value[2..];

    let mut user_info: Option<String> = None;
    let mut port: Option<u16> = None;
    let host: String;

    if let Some(user_info_sep_idx) = value.find("@") {
        user_info = value[..user_info_sep_idx].to_owned().into();
        value = &value[(user_info_sep_idx + 1)..];
    }

    // Ipv6 address
    if let Some(ipv6_start_idx) = value.find("[") {
        let ipv6_end_idx = value.find("]").ok_or(InvalidUri::InvalidHost)?;
        host = value[(ipv6_start_idx + 1)..ipv6_end_idx].to_owned().into();
        value = &value[(ipv6_end_idx + 1)..];
    }
    // Anything before the port separator is the host
    else if let Some(port_sep_idx) = value.find(":") {
        host = value[..port_sep_idx].to_owned();
        value = &value[port_sep_idx..]; // the port
    }
    // If there is not port separator everything before the "/" if the host
    else if let Some(port_sep_idx) = value.find("/") {
        host = value[..port_sep_idx].to_owned();
    }
    // Not host found
    else {
        return Err(InvalidUri::EmptyHost);
    }

    // Parse the port if any
    if let Some(port_sep_idx) = value.find(":") {
        // Parse the port
        let port_end_idx = value.find("/").unwrap_or(value.len());
        port = u16::from_str(&value[(port_sep_idx + 1)..port_end_idx])
            .map_err(|_| InvalidUri::InvalidPort)?
            .into();

        value = &value[port_end_idx..];
    }

    let authority = Authority::new(user_info, host, port);
    Ok((Some(authority), value))
}

fn parse_path_and_query(mut value: &str) -> Result<PathAndQuery, InvalidUri> {
    let path: String;
    let mut query: Option<String> = None;
    let mut fragment: Option<String> = None;

    // The fragment is the last part
    if let Some(fragment_sep_idx) = value.find("#") {
        fragment = value[(fragment_sep_idx + 1)..].to_owned().into();
        value = &value[..fragment_sep_idx];
    }

    // Before the fragment the query
    if let Some(query_sep_idx) = value.find("?") {
        query = value[(query_sep_idx + 1)..].to_owned().into();
        value = &value[..query_sep_idx];
    }

    // The last segment is the path
    path = value.to_owned();

    Ok(PathAndQuery::new(path, query, fragment))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::http::Uri;

    #[test]
    fn test_uri_with_full_components() {
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
    fn test_uri_without_user_info() {
        let uri_str =
            "https://www.example.com:1234/forum/questions/?tag=networking&order=newest#top";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert!(uri.authority().is_some());

        let authority = uri.authority().unwrap();
        assert_eq!(authority.user_info(), None);
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
    fn test_uri_without_fragment() {
        let uri_str =
            "https://john.doe@www.example.com:1234/forum/questions/?tag=networking&order=newest";
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
        assert_eq!(uri.path_and_query().fragment(), None);
    }

    #[test]
    fn test_uri_with_ipv6_host() {
        let uri_str = "ldap://[2001:db8::7]/c=GB?objectClass?one";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "ldap");
        assert!(uri.authority().is_some());
        let authority = uri.authority().unwrap();
        assert_eq!(authority.host(), "2001:db8::7");
        assert_eq!(uri.path_and_query().path(), "/c=GB");
        assert_eq!(uri.path_and_query().query(), Some("objectClass?one"));
    }

    #[test]
    fn test_uri_with_mailto_scheme() {
        let uri_str = "mailto:John.Doe@example.com";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "mailto");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path_and_query().path(), "John.Doe@example.com");
    }

    #[test]
    fn test_uri_with_tel_scheme() {
        let uri_str = "tel:+1-816-555-1212";
        let uri = Uri::from_str(uri_str).expect("Failed to parse URI");

        assert_eq!(uri.scheme().unwrap().as_str(), "tel");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path_and_query().path(), "+1-816-555-1212");
    }
}
