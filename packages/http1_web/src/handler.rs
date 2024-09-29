use http1::{body::Body, request::Request, response::Response, status::StatusCode};

use crate::{from_request::FromRequestRef, into_response::IntoResponse};

pub trait Handler<Args> {
    type Output: IntoResponse;
    fn call(&self, args: Args) -> Self::Output;
}
pub struct BoxedHandler(Box<dyn Fn(Request<Body>) -> Response<Body> + Sync + Send + 'static>);

impl BoxedHandler {
    pub fn new<H, Args, R>(handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        BoxedHandler(Box::new(move |req| match Args::from_request_ref(&req) {
            Ok(args) => {
                let result = handler.call(args);
                let res = result.into_response();
                res
            }
            Err(err) => {
                eprintln!("{err}");
                Response::new(StatusCode::INTERNAL_SERVER_ERROR, Body::empty())
            }
        }))
    }

    pub fn call(&self, req: Request<Body>) -> Response<Body> {
        (self.0)(req)
    }
}

impl<F, R> Handler<()> for F
where
    F: Fn() -> R,
    R: IntoResponse,
{
    type Output = R;

    fn call(&self, _args: ()) -> Self::Output {
        (self)()
    }
}

impl<F, R, T1> Handler<(T1,)> for F
where
    F: Fn((T1,)) -> R,
    R: IntoResponse,
{
    type Output = R;

    fn call(&self, args: (T1,)) -> Self::Output {
        (self)(args)
    }
}

impl<F, R, T1, T2> Handler<(T1, T2)> for F
where
    F: Fn((T1, T2)) -> R,
    R: IntoResponse,
{
    type Output = R;

    fn call(&self, args: (T1, T2)) -> Self::Output {
        (self)(args)
    }
}

impl<F, R, T1, T2, T3> Handler<(T1, T2, T3)> for F
where
    F: Fn((T1, T2, T3)) -> R,
    R: IntoResponse,
{
    type Output = R;

    fn call(&self, args: (T1, T2, T3)) -> Self::Output {
        (self)(args)
    }
}
