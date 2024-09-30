use std::fmt::Display;

use http1::common::date_time::DateTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cookie {
    name: String,
    value: String,
    http_only: bool,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<usize>,
    expires: Option<DateTime>,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Cookie {
            name: name.into(),
            value: value.into(),
            http_only: false,
            path: None,
            domain: None,
            max_age: None,
            expires: None,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn max_age(mut self, max_age: usize) -> Self {
        self.max_age = Some(max_age);
        self
    }

    pub fn expires(mut self, expires: impl Into<DateTime>) -> Self {
        self.expires = Some(expires.into());
        self
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Cookies {
    cookies: Vec<Cookie>,
    removed_cookies: Vec<Cookie>,
}

impl Cookies {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    pub fn set(&mut self, cookie: Cookie) {
        // Clear the cookie if is being replaced
        if let Some(pos) = self
            .removed_cookies
            .iter()
            .position(|c| c.name() == cookie.name())
        {
            self.removed_cookies.remove(pos);
        }

        // Add the new cookie
        self.cookies.push(cookie);
    }

    pub fn del(&mut self, name: impl AsRef<str>) -> Option<&Cookie> {
        match self.cookies.iter().position(|c| c.name() == name.as_ref()) {
            Some(pos) => {
                let deleted = self.cookies.remove(pos);
                self.removed_cookies.push(deleted);
                self.removed_cookies.iter().last()
            }
            None => None,
        }
    }

    /// Remove all the cookies
    pub fn clear(&mut self) {
        let all_cookies = self.cookies.drain(..);
        self.removed_cookies.extend(all_cookies);
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Cookie> {
        self.cookies.iter().find(|c| c.name() == name.as_ref())
    }

    pub fn get_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Cookie> {
        self.cookies.iter().filter(move |c| c.name() == name)
    }
}
