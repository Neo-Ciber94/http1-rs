use std::{
    borrow::Cow,
    convert::Infallible,
    fmt::{Debug, Display},
    hash::Hash,
};

use super::get_header_name;

#[derive(Debug)]
pub struct InvalidHeaderName(String);

impl std::error::Error for InvalidHeaderName {}

impl Display for InvalidHeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid header name, expected ascii string: {:?}",
            self.0
        )
    }
}

impl From<Infallible> for InvalidHeaderName {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub struct HeaderName(Cow<'static, str>);

impl HeaderName {
    pub(crate) const fn const_static(s: &'static str) -> Self {
        HeaderName(Cow::Borrowed(s))
    }

    pub fn from_static(s: &'static str) -> Self {
        Self::from_checked_static(s).unwrap()
    }

    pub fn from_string(s: String) -> Self {
        Self::from_checked_string(s).unwrap()
    }

    pub fn from_checked_static(s: &'static str) -> Result<Self, InvalidHeaderName> {
        match get_header_name(s) {
            Some(header_name) => Ok(header_name),
            None => {
                if !s.is_ascii() {
                    return Err(InvalidHeaderName(s.to_owned()));
                }

                Ok(HeaderName(Cow::Borrowed(s)))
            }
        }
    }

    pub fn from_checked_string(s: String) -> Result<Self, InvalidHeaderName> {
        match get_header_name(&s) {
            Some(header_name) => Ok(header_name),
            None => {
                if !s.is_ascii() {
                    return Err(InvalidHeaderName(s.to_owned()));
                }

                Ok(HeaderName(Cow::Owned(s)))
            }
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Eq for HeaderName {}

impl PartialEq for HeaderName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl<'a> PartialEq<&'a str> for HeaderName {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl<'a> PartialEq<&'a String> for HeaderName {
    fn eq(&self, other: &&'a String) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl Hash for HeaderName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0
            .as_bytes()
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .for_each(|c| c.hash(state))
    }
}

impl AsRef<str> for HeaderName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for HeaderName {
    type Error = InvalidHeaderName;

    fn try_from(value: String) -> Result<Self, InvalidHeaderName> {
        HeaderName::from_checked_string(value)
    }
}

impl TryFrom<&'static str> for HeaderName {
    type Error = InvalidHeaderName;

    fn try_from(value: &'static str) -> Result<Self, InvalidHeaderName> {
        HeaderName::from_checked_static(value)
    }
}

impl TryFrom<Cow<'static, str>> for HeaderName {
    type Error = InvalidHeaderName;

    fn try_from(value: Cow<'static, str>) -> Result<Self, InvalidHeaderName> {
        match value {
            Cow::Borrowed(s) => HeaderName::from_checked_static(s),
            Cow::Owned(s) => HeaderName::from_checked_string(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_from_static_str() {
        let name = HeaderName::from_static("Content-Type");
        assert_eq!(name.as_str(), "Content-Type");
    }

    #[test]
    fn should_create_from_string() {
        let name = HeaderName::from_string("Custom-Header".to_string());
        assert_eq!(name.as_str(), "Custom-Header");
    }

    #[test]
    fn should_compare_case_insensitively_with_another_header_name() {
        let name1 = HeaderName::from_static("Accept-Encoding");
        let name2 = HeaderName::from_static("accept-encoding");
        assert_eq!(name1, name2);

        let name3 = HeaderName::from_static("Content-Type");
        assert_ne!(name1, name3);
    }

    #[test]
    fn should_compare_case_insensitively_with_str_and_string() {
        let name = HeaderName::from_static("User-Agent");
        assert_eq!(name, "user-agent");
        assert_eq!(name, &"USER-AGENT".to_string());
    }
}
