use std::fmt::{Debug, Display};

/// To display an error when the expected type is different from the current value.
pub trait Expected {
    fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result;
}

macro_rules! impl_expected {
    ($($T:ty),*) => {
        $(
            impl Expected for $T {
                fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result {
                    write!(f, "expected `{expected}` but was `{}`", stringify!($T))
                }
            }
        )*
    };
}

impl_expected!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, bool, char, String);

impl<'a> Expected for &'a str {
    fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result {
        write!(f, "expected `{expected}` but was string \"{}\"", self)
    }
}

impl<'a, T> Expected for &'a [T] {
    fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result {
        write!(f, "expected `{expected}` but was slice")
    }
}

impl<T> Expected for Vec<T> {
    fn expected(&self, f: &mut std::fmt::Formatter<'_>, expected: &str) -> std::fmt::Result {
        write!(f, "expected `{expected}` but was vec")
    }
}

pub struct TypeMismatchError {
    this: Box<dyn Expected + Send + Sync + 'static>,
    expected: String,
}

impl TypeMismatchError {
    pub fn new<E, I>(this: E, expected: I) -> Self
    where
        E: Expected + Send + Sync + 'static,
        I: Into<String>,
    {
        TypeMismatchError {
            this: Box::new(this),
            expected: expected.into(),
        }
    }
}

impl Debug for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MismatchTypeError")
    }
}

impl Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.this.expected(f, &self.expected)
    }
}
