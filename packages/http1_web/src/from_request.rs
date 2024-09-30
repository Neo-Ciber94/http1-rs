use http1::{
    body::Body,
    error::BoxError,
    headers::Headers,
    method::Method,
    request::Request,
    uri::{authority::Authority, scheme::Scheme, uri::Uri},
    version::Version,
};

use crate::router::Params;

pub trait FromRequest: Sized {
    fn from_request(req: Request<Body>) -> Result<Self, BoxError>;
}

pub trait FromRequestRef: Sized {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError>;
}

impl FromRequest for Request<Body> {
    fn from_request(req: Request<Body>) -> Result<Self, BoxError> {
        Ok(req)
    }
}

impl FromRequest for Body {
    fn from_request(req: Request<Body>) -> Result<Self, BoxError> {
        Ok(req.into_body())
    }
}

impl<T> FromRequest for T
where
    T: FromRequestRef,
{
    fn from_request(req: Request<Body>) -> Result<Self, BoxError> {
        T::from_request_ref(&req)
    }
}

impl FromRequestRef for () {
    fn from_request_ref(_: &Request<Body>) -> Result<Self, BoxError> {
        Ok(())
    }
}

impl FromRequestRef for Params {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        req.extensions()
            .get::<Params>()
            .cloned()
            .ok_or_else(|| format!("Failed to get params").into())
    }
}

impl FromRequestRef for Uri {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        Ok(req.uri().clone())
    }
}

impl FromRequestRef for Scheme {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        req.uri()
            .scheme()
            .cloned()
            .ok_or_else(|| format!("Failed to get uri scheme").into())
    }
}

impl FromRequestRef for Authority {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        req.uri()
            .authority()
            .cloned()
            .ok_or_else(|| format!("Failed to get uri authority").into())
    }
}

impl FromRequestRef for Headers {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        Ok(req.headers().clone())
    }
}

impl FromRequestRef for Version {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        Ok(req.version().clone())
    }
}

impl FromRequestRef for Method {
    fn from_request_ref(req: &Request<Body>) -> Result<Self, BoxError> {
        Ok(req.method().clone())
    }
}

macro_rules! impl_from_request_tuple {
    ([$($T:ident),*], $last:ident) => {
        #[allow(non_snake_case, unused_variables)]
        impl<$($T,)* $last> crate::from_request::FromRequest for ($($T,)* $last,)
            where
                $($T: crate::from_request::FromRequestRef,)*
                $last: crate::from_request::FromRequest {
            fn from_request(req: http1::request::Request<Body>) -> Result<Self, BoxError> {
                $(
                    let $T = <$T as crate::from_request::FromRequestRef>::from_request_ref(&req)?;
                )*

                let last = <$last as crate::from_request::FromRequest>::from_request(req)?;
                Ok(($($T,)* last,))
            }
        }
    };
}

impl_from_request_tuple! { [], T1 }
impl_from_request_tuple! { [T1], T2 }
impl_from_request_tuple! { [T1, T2], T3 }
impl_from_request_tuple! { [T1, T2, T3], T4 }
impl_from_request_tuple! { [T1, T2, T3, T4], T5 }
impl_from_request_tuple! { [T1, T2, T3, T4, T5], T6 }
impl_from_request_tuple! { [T1, T2, T3, T4, T5, T6], T7 }
impl_from_request_tuple! { [T1, T2, T3, T4, T5, T6, T7], T8 }
impl_from_request_tuple! { [T1, T2, T3, T4, T5, T6, T7, T8], T9 }
impl_from_request_tuple! { [T1, T2, T3, T4, T5, T6, T7, T8, T9], T10 }
