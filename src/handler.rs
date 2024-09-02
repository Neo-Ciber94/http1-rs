use crate::http::{Request, Response};

pub trait RequestHandler {
    fn handle(&self, req: Request<String>) -> Response<String>;
}

impl<F: Fn(Request<String>) -> Response<String>> RequestHandler for F {
    fn handle(&self, req: Request<String>) -> Response<String> {
        (self)(req)
    }
}
