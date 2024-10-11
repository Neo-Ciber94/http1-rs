use http1::{
    body::Body,
    headers::Headers,
    method::Method,
    request::Request,
    response::Response,
    status::StatusCode,
    uri::{authority::Authority, path_query::PathAndQuery, scheme::Scheme, uri::Uri},
    version::Version,
};

use crate::{
    into_response::{Impossible, IntoResponse},
    router::params::ParamsMap,
};

pub trait FromRequest: Sized {
    type Rejection: IntoResponse;
    fn from_request(req: Request<Body>) -> Result<Self, Self::Rejection>;
}

pub trait FromRequestRef: Sized {
    type Rejection: IntoResponse;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection>;
}

impl<T> FromRequest for T
where
    T: FromRequestRef,
{
    type Rejection = T::Rejection;
    fn from_request(req: Request<Body>) -> Result<Self, Self::Rejection> {
        T::from_request_ref(&req)
    }
}

impl FromRequest for Request<Body> {
    type Rejection = Impossible;
    fn from_request(req: Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req)
    }
}

impl FromRequest for Body {
    type Rejection = Impossible;
    fn from_request(req: Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req.into_body())
    }
}

impl FromRequestRef for () {
    type Rejection = Impossible;
    fn from_request_ref(_: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct ParamsNotFound;
impl IntoResponse for ParamsNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        eprintln!("ParamsMap not found");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequestRef for ParamsMap {
    type Rejection = ParamsNotFound;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<ParamsMap>()
            .cloned()
            .ok_or(ParamsNotFound)
    }
}

impl FromRequestRef for Uri {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req.uri().clone())
    }
}

impl FromRequestRef for PathAndQuery {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req.uri().path_and_query().clone())
    }
}

#[doc(hidden)]
pub struct SchemeNotFound;
impl IntoResponse for SchemeNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        eprintln!("request scheme not found");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequestRef for Scheme {
    type Rejection = SchemeNotFound;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        req.uri().scheme().cloned().ok_or(SchemeNotFound)
    }
}

#[doc(hidden)]
pub struct AuthorityNotFound;
impl IntoResponse for AuthorityNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        eprintln!("request uri authority not found");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequestRef for Authority {
    type Rejection = AuthorityNotFound;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        req.uri().authority().cloned().ok_or(AuthorityNotFound)
    }
}

impl FromRequestRef for Headers {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req.headers().clone())
    }
}

impl FromRequestRef for Version {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(*req.version())
    }
}

impl FromRequestRef for Method {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(req.method().clone())
    }
}

impl<T: FromRequestRef> FromRequestRef for Option<T> {
    type Rejection = Impossible;
    fn from_request_ref(req: &Request<Body>) -> Result<Self, Self::Rejection> {
        Ok(T::from_request_ref(req).ok())
    }
}

macro_rules! impl_from_request_tuple {
    ([$($T:ident),*], $last:ident) => {
        #[allow(non_snake_case, unused_variables)]
        impl<$($T,)* $last> crate::from_request::FromRequest for ($($T,)* $last,)
            where
                $($T: crate::from_request::FromRequestRef,)*
                $last: crate::from_request::FromRequest {
            type Rejection = Response<Body>;
            fn from_request(req: http1::request::Request<Body>) -> Result<Self, Self::Rejection> {
                $(
                    let $T = match <$T as crate::from_request::FromRequestRef>::from_request_ref(&req) {
                        Ok(x) => x,
                        Err(err) => return Err(err.into_response())
                    };
                )*

                let last = match <$last as crate::from_request::FromRequest>::from_request(req) {
                    Ok(x) => x,
                    Err(err) => return Err(err.into_response())
                };

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
