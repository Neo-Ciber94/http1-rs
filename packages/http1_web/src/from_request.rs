use std::{convert::Infallible, fmt::Display};

use http1::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
    extensions::Extensions,
    headers::Headers,
    method::Method,
    payload::Payload,
    request::{Parts, Request},
    response::Response,
    status::StatusCode,
    uri::{authority::Authority, path_query::PathAndQuery, scheme::Scheme, uri::Uri},
    version::Version,
};

use crate::{routing::params::ParamsMap, IntoResponse};

pub trait FromRequest: Sized {
    type Rejection: IntoResponse;
    fn from_request(
        req: &Request<()>,
        extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection>;
}

impl FromRequest for Request<Body> {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: std::mem::take(extensions),
            headers: req.headers().clone(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: *req.version(),
        };

        Ok(Request::from_parts(
            parts,
            payload.take().unwrap_or_default(),
        ))
    }
}

impl FromRequest for Request<Payload> {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: std::mem::take(extensions),
            headers: req.headers().clone(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: *req.version(),
        };

        Ok(Request::from_parts(parts, std::mem::take(payload)))
    }
}

impl FromRequest for Request<()> {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: std::mem::take(extensions),
            headers: req.headers().clone(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: *req.version(),
        };

        Ok(Request::from_parts(parts, ()))
    }
}

impl FromRequest for Body {
    type Rejection = Infallible;

    fn from_request(
        _req: &Request<()>,
        _extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(payload.take().unwrap_or_default())
    }
}

impl FromRequest for Payload {
    type Rejection = Infallible;

    fn from_request(
        _req: &Request<()>,
        _extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(payload))
    }
}

impl FromRequest for Extensions {
    type Rejection = Infallible;

    fn from_request(
        _req: &Request<()>,
        extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(extensions))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct InvalidBodyError(BoxError);

impl Display for InvalidBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to read body: {}", self.0)
    }
}

impl IntoResponse for InvalidBodyError {
    fn into_response(self) -> Response<Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequest for Vec<u8> {
    type Rejection = InvalidBodyError;

    fn from_request(
        _req: &Request<()>,
        _extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        payload
            .take()
            .unwrap_or_default()
            .read_all_bytes()
            .map_err(InvalidBodyError)
    }
}

impl FromRequest for String {
    type Rejection = InvalidBodyError;

    fn from_request(
        _req: &Request<()>,
        _extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        payload
            .take()
            .unwrap_or_default()
            .read_all_bytes()
            .and_then(|bytes| Ok(String::from_utf8(bytes)?))
            .map_err(InvalidBodyError)
    }
}

impl FromRequest for () {
    type Rejection = Infallible;

    fn from_request(
        _req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ParamsNotFound;

impl Display for ParamsNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParamsMap not found")
    }
}

impl IntoResponse for ParamsNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequest for ParamsMap {
    type Rejection = ParamsNotFound;

    fn from_request(
        _req: &Request<()>,
        extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        extensions.get::<ParamsMap>().cloned().ok_or(ParamsNotFound)
    }
}

impl FromRequest for Uri {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(req.uri().clone())
    }
}

impl FromRequest for PathAndQuery {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(req.uri().path_and_query().clone())
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct SchemeNotFound;

impl Display for SchemeNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request scheme not found")
    }
}

impl IntoResponse for SchemeNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequest for Scheme {
    type Rejection = SchemeNotFound;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        req.uri().scheme().cloned().ok_or(SchemeNotFound)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct AuthorityNotFound;

impl Display for AuthorityNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request uri authority not found")
    }
}

impl IntoResponse for AuthorityNotFound {
    fn into_response(self) -> http1::response::Response<Body> {
        log::error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl FromRequest for Authority {
    type Rejection = AuthorityNotFound;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        req.uri().authority().cloned().ok_or(AuthorityNotFound)
    }
}

impl FromRequest for Headers {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(req.headers().clone())
    }
}

impl FromRequest for Version {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(*req.version())
    }
}

impl FromRequest for Method {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        _extensions: &mut Extensions,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(req.method().clone())
    }
}

impl<T: FromRequest> FromRequest for Option<T> {
    type Rejection = Infallible;

    fn from_request(
        req: &Request<()>,
        extensions: &mut Extensions,
        payload: &mut Payload,
    ) -> Result<Self, Self::Rejection> {
        Ok(T::from_request(req, extensions, payload).ok())
    }
}

macro_rules! impl_tuple_from_request {
    ($($T:ident),*) => {
        #[allow(non_snake_case, unused_variables)]
        impl <$($T),*> FromRequest for ($($T),*,) where $($T: FromRequest),* {
            type Rejection = Response<Body>;

            fn from_request(
                req: &Request<()>,
                extensions: &mut Extensions,
                payload: &mut Payload,
            ) -> Result<Self, Self::Rejection> {
                $(
                    let $T = match <$T as crate::from_request::FromRequest>::from_request(&req, extensions, payload) {
                        Ok(x) => x,
                        Err(err) => return Err(err.into_response())
                    };
                )*

                Ok(($($T),*,))
            }
        }
    };
}

impl_tuple_from_request!(A);
impl_tuple_from_request!(A, B);
impl_tuple_from_request!(A, B, C);
impl_tuple_from_request!(A, B, C, D);
impl_tuple_from_request!(A, B, C, D, E);
impl_tuple_from_request!(A, B, C, D, E, F);
impl_tuple_from_request!(A, B, C, D, E, F, G);
impl_tuple_from_request!(A, B, C, D, E, F, G, H);
impl_tuple_from_request!(A, B, C, D, E, F, G, H, I);
impl_tuple_from_request!(A, B, C, D, E, F, G, H, I, J);
impl_tuple_from_request!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_from_request!(A, B, C, D, E, F, G, H, I, J, K, L);

pub(crate) fn split_request(req: Request<Body>) -> (Request<()>, Extensions, Payload) {
    let (body, mut parts) = req.into_parts();
    let payload = Payload::Data(body);
    let extensions = std::mem::take(&mut parts.extensions);

    let req = Request::from_parts(parts, ());
    (req, extensions, payload)
}

pub(crate) fn join_request(
    mut req: Request<()>,
    extensions: Extensions,
    mut payload: Payload,
) -> Request<Body> {
    let body = payload.take().unwrap_or_default();
    *req.extensions_mut() = extensions;
    req.map_body(move |_| body)
}
