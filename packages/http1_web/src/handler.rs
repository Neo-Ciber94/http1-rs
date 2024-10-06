use http1::{body::Body, request::Request, response::Response, status::StatusCode};

use crate::{from_request::FromRequest, into_response::IntoResponse};

pub trait Handler<Args> {
    type Output: IntoResponse;
    fn call(&self, args: Args) -> Self::Output;
}
pub struct BoxedHandler(Box<dyn Fn(Request<Body>) -> Response<Body> + Sync + Send + 'static>);

impl BoxedHandler {
    pub fn new<H, Args, R>(handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        BoxedHandler(Box::new(move |req| match Args::from_request(req) {
            Ok(args) => {
                let result = handler.call(args);
                result.into_response()
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

macro_rules! impl_handler_for_tuple {
    ($($T:ident),*) => {
        impl<Func, R, $($T),*> Handler<($($T),*,)> for Func
        where
            Func: Fn($($T),*) -> R,
            R: IntoResponse,
        {
            type Output = R;

            #[allow(non_snake_case)]
            fn call(&self, ($($T),*,): ($($T,)*)) -> Self::Output {
                (self)($($T),*)
            }
        }
    };
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

impl_handler_for_tuple! { T1 }
impl_handler_for_tuple! { T1, T2 }
impl_handler_for_tuple! { T1, T2, T3 }
impl_handler_for_tuple! { T1, T2, T3, T4 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5, T6 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5, T6, T7 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5, T6, T7, T8 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9 }
impl_handler_for_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 }
