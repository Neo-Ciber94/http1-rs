use std::{borrow::Cow, fmt::Display, hash::Hash};

#[derive(Debug, Clone)]
pub struct HeaderName(Cow<'static, str>);

impl HeaderName {
    pub const fn from_static(s: &'static str) -> Self {
        HeaderName(Cow::Borrowed(s))
    }

    pub const fn from_string(s: String) -> Self {
        HeaderName(Cow::Owned(s))
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

impl From<String> for HeaderName {
    fn from(value: String) -> Self {
        HeaderName::from_string(value)
    }
}

impl From<&'static str> for HeaderName {
    fn from(value: &'static str) -> Self {
        HeaderName::from_static(value)
    }
}
