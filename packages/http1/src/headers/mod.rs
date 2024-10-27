#![allow(clippy::module_inception)]

mod headers;
mod name;
pub(crate) mod non_empty_list;
mod value;

pub use headers::*;
pub use name::*;
pub use value::*;

// Headers

macro_rules! define_header_names {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(
            pub const $name: HeaderName = HeaderName::const_static($value);
        )*

        pub(crate) fn get_header_name(name: &str) -> Option<HeaderName> {
            match name {
                $(
                    _ if $value.eq_ignore_ascii_case(name) => Some(HeaderName::const_static($value)),
                )*
                _ => None
            }
        }
    };
}

define_header_names! {
    CONTENT_LENGTH => "Content-Length",
    CONTENT_TYPE => "Content-Type",
    CONTENT_ENCODING => "Content-Encoding",
    TRANSFER_ENCODING => "Transfer-Encoding",
    CONTENT_DISPOSITION => "Content-Disposition",
    ACCEPT => "Accept",
    ACCEPT_ENCODING => "Accept-Encoding",
    ACCEPT_LANGUAGE => "Accept-Language",
    AUTHORIZATION => "Authorization",
    CACHE_CONTROL => "Cache-Control",
    CONNECTION => "Connection",
    COOKIE => "Cookie",
    SET_COOKIE => "Set-Cookie",
    HOST => "Host",
    USER_AGENT => "User-Agent",
    REFERER => "Referer",
    UPGRADE => "Upgrade",
    LOCATION => "Location",
    X_FORWARDED_FOR => "X-Forwarded-For",
    X_FRAME_OPTIONS => "X-Frame-Options",
    ORIGIN => "Origin",
    DATE => "Date",
    ETAG => "ETag",
    LAST_MODIFIED => "Last-Modified",
    IF_MATCH => "If-Match",
    IF_NONE_MATCH => "If-None-Match",
    IF_MODIFIED_SINCE => "If-Modified-Since",
    IF_UNMODIFIED_SINCE => "If-Unmodified-Since",
    RANGE => "Range",
    ACCEPT_RANGES => "Accept-Ranges",
    RETRY_AFTER => "Retry-After",
    VARY => "Vary",
    EXPECT => "Expect",
    ALLOW => "Allow",
    ACCESS_CONTROL_ALLOW_ORIGIN => "Access-Control-Allow-Origin",
    ACCESS_CONTROL_ALLOW_HEADERS => "Access-Control-Allow-Headers",
    ACCESS_CONTROL_ALLOW_METHODS => "Access-Control-Allow-Methods",
    ACCESS_CONTROL_MAX_AGE => "Access-Control-Max-Age",
    ACCESS_CONTROL_ALLOW_CREDENTIALS => "Access-Control-Allow-Credentials",
    ACCESS_CONTROL_EXPOSE_HEADERS => "Access-Control-Expose-Headers",
    ACCESS_CONTROL_REQUEST_HEADERS => "Access-Control-Request-Headers",
    ACCESS_CONTROL_REQUEST_METHOD => "Access-Control-Request-Method",
    CONTENT_SECURITY_POLICY => "Content-Security-Policy",
    CONTENT_SECURITY_POLICY_REPORT_ONLY => "Content-Security-Policy-Report-Only",
    FEATURE_POLICY => "Feature-Policy",
    PERMISSIONS_POLICY => "Permissions-Policy"
}
