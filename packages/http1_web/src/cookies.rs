use std::{convert::Infallible, fmt::Display, str::FromStr};

use http1::{
    body::Body,
    headers::{self, Headers},
    response::Response,
    status::StatusCode,
    uri::url_encoding::{self, Alphabet},
};

use datetime::DateTime;

use crate::{from_request::FromRequestRef, IntoResponse, IntoResponseParts};

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

pub struct SameSiteParseError;

impl FromStr for SameSite {
    type Err = SameSiteParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.eq_ignore_ascii_case("strict") => Ok(SameSite::Strict),
            _ if s.eq_ignore_ascii_case("lax") => Ok(SameSite::Lax),
            _ if s.eq_ignore_ascii_case("none") => Ok(SameSite::None),
            _ => Err(SameSiteParseError),
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
    max_age: Option<u64>,
    expires: Option<DateTime>,
    same_site: Option<SameSite>,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Builder {
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

    pub fn max_age(&self) -> Option<u64> {
        self.max_age
    }

    pub fn expires(&self) -> Option<DateTime> {
        self.expires
    }

    pub fn same_site(&self) -> Option<SameSite> {
        self.same_site
    }
}

#[derive(Debug)]
pub enum CookieParseError {
    InvalidCookie,
    InvalidMaxAge,
    InvalidExpires,
    InvalidSameSite,
}

impl std::error::Error for CookieParseError {}

impl Display for CookieParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieParseError::InvalidCookie => write!(f, "invalid cookie format"),
            CookieParseError::InvalidMaxAge => write!(f, "invalid cookie max-age"),
            CookieParseError::InvalidExpires => write!(f, "invalid cookie expires date"),
            CookieParseError::InvalidSameSite => write!(f, "invalid cookie same site"),
        }
    }
}

impl FromStr for Cookie {
    type Err = CookieParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(";").map(|x| x.trim());

        let mut builder = match parts.next().and_then(|x| x.split_once("=")) {
            Some((name, value)) => Cookie::new(name, value),
            None => return Err(CookieParseError::InvalidCookie),
        };

        for attr in parts {
            match attr {
                _ if attr.eq_ignore_ascii_case("httponly") => builder = builder.http_only(true),
                _ if attr.eq_ignore_ascii_case("secure") => builder = builder.secure(true),
                _ if attr.eq_ignore_ascii_case("partitioned") => {
                    builder = builder.partitioned(true)
                }
                _ if attr.starts_with("path=") => {
                    builder = builder.path(&attr[5..]);
                }
                _ if attr.starts_with("domain=") => {
                    builder = builder.domain(&attr[7..]);
                }
                _ if attr.starts_with("max-age=") => {
                    let max_age: u64 = attr[8..]
                        .parse()
                        .map_err(|_| CookieParseError::InvalidMaxAge)?;
                    builder = builder.max_age(max_age);
                }
                _ if attr.starts_with("expires=") => {
                    let expires = DateTime::parse_rfc_1123(&attr[8..])
                        .map_err(|_| CookieParseError::InvalidExpires)?;
                    builder = builder.expires(expires);
                }
                _ if attr.starts_with("samesite=") => {
                    let same_site: SameSite = attr[9..]
                        .parse()
                        .map_err(|_| CookieParseError::InvalidSameSite)?;
                    builder = builder.same_site(same_site);
                }
                _ => {}
            }
        }

        Ok(builder.build())
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

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = value.into();
        self
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

    pub fn max_age(mut self, max_age: u64) -> Self {
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
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#attributes
        pub struct CookieASCII;

        #[allow(clippy::match_like_matches_macro)]
        impl Alphabet for CookieASCII {
            fn contains(&self, value: u8) -> bool {
                match value {
                    b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'0'..=b'9'
                    | b'-'
                    | b'_'
                    | b'.'
                    | b'~'
                    | b'('
                    | b')'
                    | b'<'
                    | b'>'
                    | b'@'
                    | b','
                    | b';'
                    | b':'
                    | b'\\'
                    | b'"'
                    | b'/'
                    | b'['
                    | b']'
                    | b'?'
                    | b'='
                    | b'{'
                    | b'}' => true,
                    _ => false,
                }
            }
        }

        let name = url_encoding::encode_with(self.name(), CookieASCII);
        let value = url_encoding::encode_with(self.value(), CookieASCII);

        write!(f, "{name}={value}")?;

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(path) = self.path.as_deref() {
            write!(f, "; Path={}", url_encoding::encode_with(path, CookieASCII))?;
        }

        if let Some(domain) = self.domain.as_deref() {
            write!(
                f,
                "; Domain={}",
                url_encoding::encode_with(domain, CookieASCII)
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
        mut res: crate::ResponseParts,
    ) -> Result<crate::ResponseParts, Self::Err> {
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

pub type CookiesIter<'a> =
    std::iter::Chain<std::slice::Iter<'a, Cookie>, std::slice::Iter<'a, Cookie>>;

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
                let deleted = Builder(self.cookies.remove(pos))
                    .value("")
                    .expires(DateTime::with_millis(0));

                self.removed_cookies.push(deleted.into());
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

    pub fn iter(&self) -> CookiesIter<'_> {
        self.cookies.iter().chain(self.removed_cookies.iter())
    }
}

impl<'a> IntoIterator for &'a Cookies {
    type Item = &'a Cookie;
    type IntoIter = CookiesIter<'a>;

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
        // Set any new cookies
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
        mut res: crate::ResponseParts,
    ) -> Result<crate::ResponseParts, Self::Err> {
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

impl FromRequestRef for Cookies {
    type Rejection = Infallible;

    fn from_request_ref(req: &http1::request::Request<Body>) -> Result<Self, Self::Rejection> {
        let values = req.headers().get_all(headers::COOKIE);
        let mut cookies = Cookies::new();

        for header_value in values {
            let raw = header_value.as_str();
            match Cookie::from_str(raw) {
                Ok(cookie) => {
                    cookies.set(cookie);
                }
                Err(err) => {
                    log::warn!("Failed to parse cookie: `{raw}`: {err}");
                }
            }
        }

        Ok(cookies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datetime::Month;

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
