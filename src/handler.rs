use crate::{body::Body, http::{Request, Response}};

pub trait RequestHandler {
    fn handle(&self, req: Request<Body>) -> Response<Body>;
}

impl<F: Fn(Request<Body>) -> Response<Body>> RequestHandler for F {
    fn handle(&self, req: Request<Body>) -> Response<Body> {
        (self)(req)
    }
}
