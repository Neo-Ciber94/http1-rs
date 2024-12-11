/// Provides a default server handler for `http1`.
pub mod app;

/// Client ip extractors.
pub mod client_ip;

/// Connection info extractors.
pub mod conn_info;

/// Cookies response and extractors.
pub mod cookies;

/// Form extractors.
pub mod forms;

/// Request extractors.
pub mod from_request;

/// File responses.
pub mod fs;

/// Router request handler.
pub mod handler;

/// Headers extractor.
pub mod header;

/// HTML response.
pub mod html;

/// JSON response and extractor.
pub mod json;

/// Middleware handler.
pub mod middleware;

/// Mime type utilities.
pub mod mime;

/// Request path extractor.
pub mod path;

/// Request query extractor.
pub mod query;

/// Redirect response.
pub mod redirect;

/// Server router.
pub mod routing;

/// Server info extractor.
pub mod server_info;

/// App state extractor.
pub mod state;

/// Websocket extractor and utilities.
pub mod ws;

mod request;
mod response;
pub use {request::*, response::*};

mod macros {
    /// Checks a `Result<T, impl IntoResponse>` and returns the error response if is an error.
    #[macro_export]
    macro_rules! guard {
        ($expr:expr) => {
            match $expr {
                Ok(res) => res,
                Err(err) => {
                    return $crate::IntoResponse::into_response(err);
                }
            }
        };
    }
}
