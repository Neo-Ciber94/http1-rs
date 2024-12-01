use std::{fmt::Debug, sync::Arc};

use http1::{body::Body, request::Request, response::Response};

use crate::{from_request::FromRequest, IntoResponse};

pub trait Handler<Args> {
    type Output: IntoResponse;
    fn call(&self, args: Args) -> Self::Output;
}

#[derive(Clone)]
pub struct BoxedHandler {
    inner: Arc<dyn Fn(Request<Body>) -> Response<Body> + Sync + Send + 'static>,

    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl Debug for BoxedHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            f.debug_tuple("BoxedHandler")
                .field(&self.type_name)
                .finish()
        } else {
            f.debug_struct("BoxedHandler").finish_non_exhaustive()
        }
    }
}

impl Eq for BoxedHandler {}

impl PartialEq for BoxedHandler {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl BoxedHandler {
    pub fn new<H, Args, R>(handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        BoxedHandler {
            inner: Arc::new(move |req| match Args::from_whole_request(req) {
                Ok(args) => {
                    let result = handler.call(args);
                    result.into_response()
                }
                Err(err) => err.into_response(),
            }),

            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<H>(),
        }
    }

    pub fn call(&self, req: Request<Body>) -> Response<Body> {
        (self.inner)(req)
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
