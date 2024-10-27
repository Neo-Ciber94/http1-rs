use std::fmt::{Debug, Display};

use super::{de::Unexpected, visitor::Visitor};

/// To display an error when the expected type is different from the current value.
pub trait Expected {
    fn expected(&self) -> &'static str;
}

impl<T> Expected for T
where
    T: Visitor + ?Sized,
{
    fn expected(&self) -> &'static str {
        T::expected(self)
    }
}

impl Expected for &'static str {
    fn expected(&self) -> &'static str {
        self
    }
}

#[derive(Debug)]
pub struct TypeMismatchError {
    unexpected: Unexpected,
    expected: &'static str,
}

impl TypeMismatchError {
    pub fn new<T>(unexpected: Unexpected, expected: T) -> Self
    where
        T: Expected,
    {
        TypeMismatchError {
            unexpected,
            expected: expected.expected(),
        }
    }
}

impl Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "unexpected `{}`, expected `{}`",
            self.unexpected, self.expected
        )
    }
}
