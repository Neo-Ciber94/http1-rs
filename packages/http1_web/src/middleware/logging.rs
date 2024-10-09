/// A middleware that logs each request.
pub struct Logging;

impl Middleware for Logging {
    fn on_request(&self, req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
        let method = req.method().clone();
        let uri = req.uri().to_string();
        let start_time = DateTime::now_utc();

        let response = next.call(req);
        let status_code = response.status();
        let duration_ms = DateTime::now_utc().millis_since(start_time);
        let date = start_time.to_iso_8601_string();
        println!("[{date}] {method} {uri} - {status_code} - {duration_ms}ms");

        response
    }
}
