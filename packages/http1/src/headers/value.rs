use std::{borrow::Cow, convert::Infallible, fmt::Display};

#[derive(Debug)]
pub struct InvalidHeaderValue(String);

impl std::error::Error for InvalidHeaderValue {}

impl Display for InvalidHeaderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid header value, expected ascii string: {:?}",
            self.0
        )
    }
}

impl From<Infallible> for InvalidHeaderValue {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeaderValue(Cow<'static, str>);

impl HeaderValue {
    pub fn from_static(s: &'static str) -> Self {
        Self::from_checked_static(s).unwrap()
    }

    pub fn from_string(s: String) -> Self {
        Self::from_checked_string(s).unwrap()
    }

    pub fn from_checked_static(s: &'static str) -> Result<Self, InvalidHeaderValue> {
        if !s.is_ascii() {
            return Err(InvalidHeaderValue(s.to_owned()));
        }

        Ok(HeaderValue(Cow::Borrowed(s)))
    }

    pub fn from_checked_string(s: String) -> Result<Self, InvalidHeaderValue> {
        if !s.is_ascii() {
            return Err(InvalidHeaderValue(s));
        }

        Ok(HeaderValue(Cow::Owned(s)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for HeaderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for HeaderValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a str> for HeaderValue {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> PartialEq<&'a str> for &'a HeaderValue {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

macro_rules! impl_from_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl From<$type> for HeaderValue {
                fn from(value: $type) -> Self {
                    HeaderValue(Cow::Owned(value.to_string()))
                }
            }
        )*
    };
}

impl_from_value! { u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64 }

impl TryFrom<String> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        HeaderValue::from_checked_string(value)
    }
}

impl TryFrom<&'static str> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        HeaderValue::from_checked_static(value)
    }
}

impl TryFrom<Cow<'static, str>> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: Cow<'static, str>) -> Result<Self, Self::Error> {
        match value {
            Cow::Borrowed(s) => HeaderValue::from_checked_static(s),
            Cow::Owned(s) => HeaderValue::from_checked_string(s),
        }
    }
}
