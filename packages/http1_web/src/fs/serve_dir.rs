use std::path::PathBuf;

use http1::{body::Body, request::Request, response::Response};

use crate::handler::Handler;

#[derive(Debug)]
pub struct ServeDir {
    from: PathBuf,
}

impl Handler<Request<Body>> for ServeDir {
    type Output = Response<Body>;

    fn call(&self, req: Request<Body>) -> Self::Output {
        todo!()
    }
}
