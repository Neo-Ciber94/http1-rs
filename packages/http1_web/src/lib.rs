pub mod app;
pub mod client_ip;
pub mod conn_info;
pub mod cookies;
pub mod forms;
pub mod from_request;
pub mod fs;
pub mod handler;
pub mod header;
pub mod html;
pub mod json;
pub mod middleware;
pub mod mime;
pub mod path;
pub mod query;
pub mod redirect;
pub mod routing;
pub mod server_info;
pub mod state;
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
