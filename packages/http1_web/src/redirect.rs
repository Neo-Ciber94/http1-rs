use http1::{
    body::Body,
    headers::{self, HeaderValue},
    response::Response,
    status::StatusCode,
};

use crate::IntoResponse;

/// Represents the status codes for different types of redirections.
///
/// # Variants
///
/// - `MovedPermanently`: Corresponds to HTTP status 301.
/// - `Found`: Corresponds to HTTP status 302.
/// - `SeeOther`: Corresponds to HTTP status 303.
/// - `TemporaryRedirect`: Corresponds to HTTP status 307.
/// - `PermanentRedirect`: Corresponds to HTTP status 308.
/// - `NotModified`: Corresponds to HTTP status 304 (used for caching purposes).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RedirectionStatus {
    MovedPermanently,
    Found,
    SeeOther,
    TemporaryRedirect,
    PermanentRedirect,
    NotModified,
}

/// Represents an HTTP redirect response.
///
/// This struct contains the HTTP status code and the location to which the client
/// should be redirected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    status: RedirectionStatus,
    location: String,
}

impl Redirect {
    /// Creates a new `Redirect` with the given status and location.
    ///
    /// # Arguments
    ///
    /// * `status` - The type of redirection to perform (e.g., `MovedPermanently`, `Found`).
    /// * `location` - The URL to which the client should be redirected.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` struct containing the provided status and location.
    pub fn new(status: RedirectionStatus, location: impl Into<String>) -> Self {
        let location = location.into();
        Redirect { status, location }
    }

    /// Creates a `Redirect` for a 301 Moved Permanently status.
    ///
    /// # Arguments
    ///
    /// * `location` - The new permanent URL where the resource has been moved to.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 301.
    pub fn moved_permanently(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::MovedPermanently, location)
    }

    /// Creates a `Redirect` for a 302 Found status.
    ///
    /// # Arguments
    ///
    /// * `location` - The temporary URL to which the client should be redirected.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 302.
    pub fn found(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::Found, location)
    }

    /// Creates a `Redirect` for a 303 See Other status.
    ///
    /// # Arguments
    ///
    /// * `location` - The URL for a "see other" redirection.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 303.
    pub fn see_other(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::SeeOther, location)
    }

    /// Creates a `Redirect` for a 307 Temporary Redirect status.
    ///
    /// # Arguments
    ///
    /// * `location` - The temporary URL to which the client should be redirected.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 307.
    pub fn temporary_redirect(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::TemporaryRedirect, location)
    }

    /// Creates a `Redirect` for a 308 Permanent Redirect status.
    ///
    /// # Arguments
    ///
    /// * `location` - The new permanent URL where the resource has been moved to.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 308.
    pub fn permanent_redirect(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::PermanentRedirect, location)
    }

    /// Creates a `Redirect` for a 304 Not Modified status.
    ///
    /// # Arguments
    ///
    /// * `location` - The URL for caching purposes where the resource is not modified.
    ///
    /// # Returns
    ///
    /// Returns a `Redirect` with status 304.
    pub fn not_modified(location: impl Into<String>) -> Self {
        Self::new(RedirectionStatus::NotModified, location)
    }

    /// Returns the HTTP status code corresponding to the redirection type.
    ///
    /// # Returns
    ///
    /// Returns a `StatusCode` based on the type of redirection.
    pub fn status_code(&self) -> StatusCode {
        match self.status {
            RedirectionStatus::MovedPermanently => StatusCode::MOVED_PERMANENTLY,
            RedirectionStatus::Found => StatusCode::FOUND,
            RedirectionStatus::SeeOther => StatusCode::SEE_OTHER,
            RedirectionStatus::TemporaryRedirect => StatusCode::TEMPORARY_REDIRECT,
            RedirectionStatus::PermanentRedirect => StatusCode::PERMANENT_REDIRECT,
            RedirectionStatus::NotModified => StatusCode::NOT_MODIFIED,
        }
    }

    /// Returns the location URL for the redirect.
    ///
    /// # Returns
    ///
    /// A reference to the string containing the location.
    pub fn location(&self) -> &str {
        &self.location
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let status_code = self.status_code();
        let location = self.location;

        Response::builder()
            .insert_header(headers::LOCATION, HeaderValue::from_string(location))
            .status(status_code)
            .body(Body::empty())
    }
}
