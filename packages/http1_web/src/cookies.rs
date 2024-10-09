use std::{convert::Infallible, fmt::Display};

use http1::{
    body::Body,
    common::date_time::DateTime,
    headers::{self, Headers},
    response::Response,
    status::StatusCode,
    uri::convert::{self, CookieCharset},
};

use crate::into_response::{IntoResponse, IntoResponseParts};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cookie {
    name: String,
    value: String,
    http_only: bool,
    secure: bool,
    partitioned: bool,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<usize>,
    expires: Option<DateTime>,
    same_site: Option<SameSite>,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Cookie {
            name: name.into(),
            value: value.into(),
            http_only: false,
            secure: false,
            partitioned: false,
            path: None,
            domain: None,
            max_age: None,
            expires: None,
            same_site: None,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn is_http_only(&self) -> bool {
        self.http_only
    }

    pub fn is_secure(&self) -> bool {
        self.secure
    }

    pub fn is_partitioned(&self) -> bool {
        self.partitioned
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    pub fn max_age(&self) -> Option<usize> {
        self.max_age
    }

    pub fn expires(&self) -> Option<DateTime> {
        self.expires
    }

    pub fn same_site(&self) -> Option<SameSite> {
        self.same_site
    }
}

#[derive(Debug, Clone)]
pub struct Builder(Cookie);

impl Builder {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Builder(Cookie {
            name: name.into(),
            value: value.into(),
            http_only: false,
            secure: false,
            partitioned: false,
            path: None,
            domain: None,
            max_age: None,
            expires: None,
            same_site: None,
        })
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.0.http_only = http_only;
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.0.secure = secure;
        self
    }

    pub fn partitioned(mut self, partitioned: bool) -> Self {
        self.0.partitioned = partitioned;
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.0.path = Some(path.into());
        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.0.domain = Some(domain.into());
        self
    }

    pub fn max_age(mut self, max_age: usize) -> Self {
        self.0.max_age = Some(max_age);
        self
    }

    pub fn expires(mut self, expires: impl Into<DateTime>) -> Self {
        self.0.expires = Some(expires.into());
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.0.same_site = Some(same_site);
        self
    }

    pub fn build(self) -> Cookie {
        self.0
    }
}

impl From<Builder> for Cookie {
    fn from(value: Builder) -> Self {
        value.build()
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}={value}",
            name = convert::encode_uri_component_with(self.name(), CookieCharset),
            value = convert::encode_uri_component_with(self.value(), CookieCharset),
        )?;

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(path) = self.path.as_deref() {
            write!(
                f,
                "; Path={}",
                convert::encode_uri_component_with(path, CookieCharset)
            )?;
        }

        if let Some(domain) = self.domain.as_deref() {
            write!(
                f,
                "; Domain={}",
                convert::encode_uri_component_with(domain, CookieCharset)
            )?;
        }

        if let Some(max_age) = self.max_age {
            write!(f, "; Max-Age={max_age}")?;
        }

        if let Some(expires) = self.expires {
            write!(f, "; Expires={}", expires.to_rfc_1123_string())?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.partitioned {
            write!(f, "; Partitioned")?;
        }

        Ok(())
    }
}

impl IntoResponseParts for Cookie {
    type Err = Infallible;

    fn into_response_parts(
        self,
        mut res: crate::into_response::ResponseParts,
    ) -> Result<crate::into_response::ResponseParts, Self::Err> {
        res.headers_mut()
            .append(headers::SET_COOKIE, self.to_string());
        Ok(res)
    }
}

impl IntoResponse for Cookie {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let mut response = Response::new(StatusCode::OK, Body::empty());
        response
            .headers_mut()
            .append(headers::SET_COOKIE, self.to_string());
        response
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

    pub fn set(&mut self, cookie: impl Into<Cookie>) {
        let cookie = cookie.into();

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

    pub fn replace(&mut self, cookie: impl Into<Cookie>) {
        let cookie = cookie.into();

        self.cookies.retain(|c| c.name() != cookie.name());
        self.set(cookie)
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

    pub fn iter(&self) -> std::slice::Iter<'_, Cookie> {
        self.cookies.iter()
    }
}

impl<'a> IntoIterator for &'a Cookies {
    type Item = &'a Cookie;
    type IntoIter = std::slice::Iter<'a, Cookie>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Cookies {
    type Item = Cookie;
    type IntoIter = std::vec::IntoIter<Cookie>;

    fn into_iter(self) -> Self::IntoIter {
        self.cookies.into_iter()
    }
}

impl Extend<Cookie> for Headers {
    fn extend<T: IntoIterator<Item = Cookie>>(&mut self, iter: T) {
        for cookie in iter {
            self.append(headers::SET_COOKIE, cookie.to_string());
        }
    }
}

impl From<Cookies> for Headers {
    fn from(value: Cookies) -> Self {
        let mut headers = Headers::new();
        headers.extend(value);
        headers
    }
}

impl IntoResponseParts for Cookies {
    type Err = Infallible;

    fn into_response_parts(
        self,
        mut res: crate::into_response::ResponseParts,
    ) -> Result<crate::into_response::ResponseParts, Self::Err> {
        res.headers_mut().extend(self);
        Ok(res)
    }
}

impl IntoResponse for Cookies {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let mut response = Response::new(StatusCode::OK, Body::empty());
        response.headers_mut().extend(self);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http1::common::date_time::Month;

    #[test]
    fn should_create_new_cookie_with_name_and_value() {
        let cookie = Builder::new("session_id", "abc123").build();
        assert_eq!(cookie.name(), "session_id");
        assert_eq!(cookie.value(), "abc123");
        assert_eq!(cookie.to_string(), "session_id=abc123");
    }

    #[test]
    fn should_create_cookie_with_http_only_and_secure_flags() {
        let cookie = Builder::new("session_id", "abc123")
            .http_only(true)
            .secure(true)
            .build();

        assert_eq!(cookie.is_http_only(), true);
        assert_eq!(cookie.is_secure(), true);
        assert_eq!(cookie.to_string(), "session_id=abc123; HttpOnly; Secure");
    }

    #[test]
    fn should_create_cookie_with_path_and_domain() {
        let cookie = Builder::new("session_id", "abc123")
            .path("/app")
            .domain("example.com")
            .build();

        assert_eq!(cookie.path(), Some("/app"));
        assert_eq!(cookie.domain(), Some("example.com"));
        assert_eq!(
            cookie.to_string(),
            "session_id=abc123; Path=/app; Domain=example.com"
        );
    }

    #[test]
    fn should_create_cookie_with_max_age_and_expires() {
        let expires = DateTime::builder()
            .year(2024)
            .month(Month::December)
            .day(31)
            .hours(23)
            .minutes(59)
            .secs(59)
            .build();

        let cookie = Builder::new("session_id", "abc123")
            .max_age(3600)
            .expires(expires)
            .build();

        assert_eq!(cookie.max_age(), Some(3600));
        assert_eq!(cookie.expires(), Some(expires));
        assert_eq!(
            cookie.to_string(),
            format!(
                "session_id=abc123; Max-Age=3600; Expires={}",
                expires.to_rfc_1123_string()
            )
        );
    }

    #[test]
    fn should_create_cookie_with_partitioned() {
        let cookie = Builder::new("session_id", "abc123")
            .partitioned(true)
            .build();

        assert_eq!(cookie.is_partitioned(), true);
        assert_eq!(cookie.to_string(), "session_id=abc123; Partitioned");
    }

    #[test]
    fn should_encode_special_characters_in_cookie_name_and_value() {
        let cookie = Builder::new("special_cookie", "value with spaces").build();
        assert_eq!(cookie.to_string(), "special_cookie=value%20with%20spaces");
    }
}

#[cfg(test)]
mod cookies_tests {
    use super::*;

    #[test]
    fn should_create_empty_cookies() {
        let cookies = Cookies::new();
        assert!(cookies.is_empty());
        assert_eq!(cookies.len(), 0);
    }

    #[test]
    fn should_add_cookie() {
        let mut cookies = Cookies::new();
        let cookie = Builder::new("session_id", "abc123").build();

        cookies.set(cookie.clone());

        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies.get("session_id").unwrap(), &cookie);
    }

    #[test]
    fn should_replace_cookie_with_same_name() {
        let mut cookies = Cookies::new();
        let cookie1 = Builder::new("session_id", "abc123").build();
        let cookie2 = Builder::new("session_id", "xyz789").build();

        cookies.set(cookie1.clone());
        cookies.replace(cookie2.clone());

        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies.get("session_id").unwrap(), &cookie2); // Cookie should be replaced with `cookie2`
    }

    #[test]
    fn should_delete_cookie() {
        let mut cookies = Cookies::new();
        let cookie = Builder::new("session_id", "abc123").build();

        cookies.set(cookie.clone());

        let deleted_cookie = cookies.del("session_id").unwrap();

        assert_eq!(deleted_cookie.name(), "session_id");
        assert!(cookies.is_empty());
        assert_eq!(cookies.len(), 0);
    }

    #[test]
    fn should_return_none_when_deleting_non_existent_cookie() {
        let mut cookies = Cookies::new();
        assert!(cookies.del("non_existent").is_none());
    }

    #[test]
    fn should_clear_all_cookies() {
        let mut cookies = Cookies::new();
        cookies.set(Builder::new("cookie1", "value1").build());
        cookies.set(Builder::new("cookie2", "value2").build());

        assert_eq!(cookies.len(), 2);
        cookies.clear();

        assert!(cookies.is_empty());
        assert_eq!(cookies.len(), 0);
    }

    #[test]
    fn should_retrieve_cookie_by_name() {
        let mut cookies = Cookies::new();
        let cookie = Builder::new("session_id", "abc123").build();

        cookies.set(cookie.clone());

        let retrieved_cookie = cookies.get("session_id").unwrap();

        assert_eq!(retrieved_cookie, &cookie);
    }

    #[test]
    fn should_return_none_for_non_existent_cookie() {
        let cookies = Cookies::new();
        assert!(cookies.get("non_existent").is_none());
    }

    #[test]
    fn should_get_all_cookies_with_same_name() {
        let mut cookies = Cookies::new();
        let cookie1 = Builder::new("session_id", "abc123").build();
        let cookie2 = Builder::new("session_id", "xyz789").build();

        cookies.set(cookie1.clone());
        cookies.set(cookie2.clone());

        let all_cookies: Vec<&Cookie> = cookies.get_all("session_id").collect();

        assert_eq!(all_cookies.len(), 2);
        assert!(all_cookies.contains(&&cookie1));
        assert!(all_cookies.contains(&&cookie2));
    }

    #[test]
    fn should_iterate_over_all_cookies() {
        let mut cookies = Cookies::new();
        let cookie1 = Builder::new("cookie1", "value1").build();
        let cookie2 = Builder::new("cookie2", "value2").build();

        cookies.set(cookie1.clone());
        cookies.set(cookie2.clone());

        let mut iter = cookies.iter();
        assert_eq!(iter.next().unwrap(), &cookie1);
        assert_eq!(iter.next().unwrap(), &cookie2);
        assert!(iter.next().is_none());
    }
}
