mod entry;
mod headers;
mod name;

pub use headers::*;
pub use name::*;

// Headers

pub const CONTENT_LENGTH: HeaderName = HeaderName::from_static("Content-Length");

pub const CONTENT_TYPE: HeaderName = HeaderName::from_static("Content-Type");

pub const CONTENT_ENCODING: HeaderName = HeaderName::from_static("Content-Encoding");

pub const TRANSFER_ENCODING: HeaderName = HeaderName::from_static("Transfer-Encoding");

pub const CONTENT_DISPOSITION: HeaderName = HeaderName::from_static("Content-Disposition");

pub const ACCEPT: HeaderName = HeaderName::from_static("Accept");

pub const ACCEPT_ENCODING: HeaderName = HeaderName::from_static("Accept-Encoding");

pub const ACCEPT_LANGUAGE: HeaderName = HeaderName::from_static("Accept-Language");

pub const AUTHORIZATION: HeaderName = HeaderName::from_static("Authorization");

pub const CACHE_CONTROL: HeaderName = HeaderName::from_static("Cache-Control");

pub const CONNECTION: HeaderName = HeaderName::from_static("Connection");

pub const COOKIE: HeaderName = HeaderName::from_static("Cookie");

pub const SET_COOKIE: HeaderName = HeaderName::from_static("Set-Cookie");

pub const HOST: HeaderName = HeaderName::from_static("Host");

pub const USER_AGENT: HeaderName = HeaderName::from_static("User-Agent");

pub const REFERER: HeaderName = HeaderName::from_static("Referer");

pub const UPGRADE: HeaderName = HeaderName::from_static("Upgrade");

pub const LOCATION: HeaderName = HeaderName::from_static("Location");

pub const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("X-Forwarded-For");

pub const X_FRAME_OPTIONS: HeaderName = HeaderName::from_static("X-Frame-Options");

pub const ORIGIN: HeaderName = HeaderName::from_static("Origin");

pub const DATE: HeaderName = HeaderName::from_static("Date");

pub const ETAG: HeaderName = HeaderName::from_static("ETag");

pub const LAST_MODIFIED: HeaderName = HeaderName::from_static("Last-Modified");

pub const IF_MATCH: HeaderName = HeaderName::from_static("If-Match");

pub const IF_NONE_MATCH: HeaderName = HeaderName::from_static("If-None-Match");

pub const IF_MODIFIED_SINCE: HeaderName = HeaderName::from_static("If-Modified-Since");

pub const IF_UNMODIFIED_SINCE: HeaderName = HeaderName::from_static("If-Unmodified-Since");

pub const RANGE: HeaderName = HeaderName::from_static("Range");

pub const ACCEPT_RANGES: HeaderName = HeaderName::from_static("Accept-Ranges");

pub const RETRY_AFTER: HeaderName = HeaderName::from_static("Retry-After");

pub const VARY: HeaderName = HeaderName::from_static("Vary");

pub const EXPECT: HeaderName = HeaderName::from_static("Expect");

pub const ALLOW: HeaderName = HeaderName::from_static("Allow");

pub const ACCESS_CONTROL_ALLOW_ORIGIN: HeaderName =
    HeaderName::from_static("Access-Control-Allow-Origin");

pub const ACCESS_CONTROL_ALLOW_HEADERS: HeaderName =
    HeaderName::from_static("Access-Control-Allow-Headers");

pub const ACCESS_CONTROL_ALLOW_METHODS: HeaderName =
    HeaderName::from_static("Access-Control-Allow-Methods");
pub const ACCESS_CONTROL_MAX_AGE: HeaderName = HeaderName::from_static("Access-Control-Max-Age");

pub const ACCESS_CONTROL_EXPOSE_HEADERS: HeaderName =
    HeaderName::from_static("Access-Control-Expose-Headers");

pub const ACCESS_CONTROL_REQUEST_HEADERS: HeaderName =
    HeaderName::from_static("Access-Control-Request-Headers");

pub const ACCESS_CONTROL_REQUEST_METHOD: HeaderName =
    HeaderName::from_static("Access-Control-Request-Method");
