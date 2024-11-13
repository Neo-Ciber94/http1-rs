use std::{borrow::Cow, fmt::Display};

#[derive(Default, Debug, Clone, Hash, PartialOrd, Ord)]
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

impl Eq for HeaderValue {}

impl PartialEq for HeaderValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl<'a> PartialEq<&'a str> for HeaderValue {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl<'a> PartialEq<&'a String> for HeaderValue {
    fn eq(&self, other: &&'a String) -> bool {
        self.0.eq_ignore_ascii_case(other)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_from_different_sources() {
        assert_eq!(
            HeaderValue::from_static("static_value").as_str(),
            "static_value"
        );
        assert_eq!(
            HeaderValue::from("owned_value".to_string()).as_str(),
            "owned_value"
        );
        assert_eq!(HeaderValue::from(123).as_str(), "123");
        assert_eq!(HeaderValue::from(45.67).as_str(), "45.67");
    }

    #[test]
    fn should_compare_case_insensitively_with_another_header_value() {
        let value1 = HeaderValue::from_static("Test");
        let value2 = HeaderValue::from_static("test");
        assert_eq!(value1, value2);

        let value3 = HeaderValue::from_static("Example");
        assert_ne!(value1, value3);
    }

    #[test]
    fn should_compare_case_insensitively_with_str_and_string() {
        let value = HeaderValue::from_static("Test");
        assert_eq!(value, "test");
        assert_eq!(value, &"test".to_string());
    }
}
