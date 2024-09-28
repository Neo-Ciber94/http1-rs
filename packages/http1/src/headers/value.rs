use std::{borrow::Cow, fmt::Display};

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeaderValue(Cow<'static, str>);

impl HeaderValue {
    pub fn from_static(s: &'static str) -> Self {
        HeaderValue(Cow::Borrowed(s))
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

impl From<String> for HeaderValue {
    fn from(value: String) -> Self {
        HeaderValue(Cow::Owned(value))
    }
}

impl From<&'static str> for HeaderValue {
    fn from(value: &'static str) -> Self {
        HeaderValue::from_static(value)
    }
}
