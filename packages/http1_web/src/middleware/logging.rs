use crate::handler::BoxedHandler;
use crate::middleware::Middleware;
use datetime::DateTime;
use http1::body::Body;
use http1::request::Request;
use http1::response::Response;
/// A middleware that logs each request.
pub struct Logging;

impl Middleware for Logging {
    fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        let method = req.method().clone();
        let uri = req.uri().to_string();
        let now = DateTime::now_utc();

        log::info!("Request: {method} {uri}");

        let response = next.call(req);
        let status_code = response.status();
        let duration_ms = DateTime::now_utc().millis_since(now);

        let level = if status_code.is_client_error() {
            log::LogLevel::Warn
        } else if status_code.is_server_error() {
            log::LogLevel::Error
        } else {
            log::LogLevel::Info
        };

        log::log!(level, "Response: {status_code} - {duration_ms}ms");

        response
    }
}
