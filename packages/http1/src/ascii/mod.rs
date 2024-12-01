mod char;
pub use char::*;

use core::str;
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

#[derive(Debug)]
pub struct InvalidAsciiStr(String);

impl std::error::Error for InvalidAsciiStr {}

impl Display for InvalidAsciiStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid ascii string: {:?}", self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsciiString(String);

impl AsciiString {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_string(s: String) -> Result<Self, InvalidAsciiStr> {
        if s.is_ascii() {
            Ok(AsciiString(s))
        } else {
            Err(InvalidAsciiStr(s))
        }
    }

    pub fn from_ascii(s: AsciiStr<'_>) -> Self {
        AsciiString(s.into_string())
    }

    pub fn push(&mut self, ch: AsciiChar) {
        self.0.push(ch.to_char());
    }

    pub fn push_str(&mut self, s: AsciiStr<'_>) {
        self.0.push_str(&s);
    }

    pub fn insert(&mut self, idx: usize, ch: AsciiChar) {
        self.0.insert(idx, ch.to_char());
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

impl FromStr for AsciiString {
    type Err = InvalidAsciiStr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = AsciiStr::new(s)?;
        Ok(AsciiString::from_ascii(s))
    }
}

impl TryFrom<String> for AsciiString {
    type Error = InvalidAsciiStr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        AsciiString::from_string(value)
    }
}

impl<'a> TryFrom<&'a str> for AsciiString {
    type Error = InvalidAsciiStr;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let s = value.try_into()?;
        Ok(AsciiString::from_ascii(s))
    }
}

impl<'a> TryFrom<Cow<'a, str>> for AsciiString {
    type Error = InvalidAsciiStr;

    fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
        let s = AsciiStr::new(&value)?;
        Ok(AsciiString::from_ascii(s))
    }
}

impl Deref for AsciiString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for AsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for AsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsciiStr<'a>(&'a str);

impl<'a> AsciiStr<'a> {
    pub fn new(s: &'a str) -> Result<Self, InvalidAsciiStr> {
        if s.is_ascii() {
            Ok(AsciiStr(s))
        } else {
            Err(InvalidAsciiStr(s.to_owned()))
        }
    }

    pub fn from_string(s: &'a String) -> Result<Self, InvalidAsciiStr> {
        Self::new(s.as_str())
    }

    pub fn into_string(self) -> String {
        self.0.to_owned()
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl<'a> Deref for AsciiStr<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Debug for AsciiStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'a> Display for AsciiStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> TryFrom<&'a str> for AsciiStr<'a> {
    type Error = InvalidAsciiStr;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        AsciiStr::new(value)
    }
}
