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

/// Allow to create a type from an incoming request.
pub trait FromRequest: Sized {
    type Rejection: IntoResponse;

    /// Creates this value from the request parts.
    ///
    /// # Parameters
    /// - `req`: The request (without the body or extensions)
    /// - `payload`: The body
    fn from_request(req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection>;

    /// Creates this value using the request.
    fn from_whole_request(req: Request<Body>) -> Result<Self, Self::Rejection> {
        let (req, body) = req.drop_body();
        Self::from_request(&req, &mut Payload::Data(body))
    }
}

impl FromRequest for Request<Body> {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: req.extensions().clone(),
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

    fn from_request(req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: req.extensions().clone(),
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

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        let parts = Parts {
            extensions: req.extensions().clone(),
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

    fn from_request(_req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(payload.take().unwrap_or_default())
    }
}

impl FromRequest for Payload {
    type Rejection = Infallible;

    fn from_request(_req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(payload))
    }
}

impl FromRequest for Extensions {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(req.extensions().clone())
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

    fn from_request(_req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        payload
            .take()
            .unwrap_or_default()
            .read_all_bytes()
            .map_err(InvalidBodyError)
    }
}

impl FromRequest for String {
    type Rejection = InvalidBodyError;

    fn from_request(_req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
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

    fn from_request(_req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
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

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<ParamsMap>()
            .cloned()
            .ok_or(ParamsNotFound)
    }
}

impl FromRequest for Uri {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(req.uri().clone())
    }
}

impl FromRequest for PathAndQuery {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
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

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
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

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        req.uri().authority().cloned().ok_or(AuthorityNotFound)
    }
}

impl FromRequest for Headers {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(req.headers().clone())
    }
}

impl FromRequest for Version {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(*req.version())
    }
}

impl FromRequest for Method {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, _payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(req.method().clone())
    }
}

impl<T: FromRequest> FromRequest for Option<T> {
    type Rejection = Infallible;

    fn from_request(req: &Request<()>, payload: &mut Payload) -> Result<Self, Self::Rejection> {
        Ok(T::from_request(req, payload).ok())
    }
}

macro_rules! impl_tuple_from_request {
    ($($T:ident),*) => {
        #[allow(non_snake_case, unused_variables)]
        impl <$($T),*> FromRequest for ($($T),*,) where $($T: FromRequest),* {
            type Rejection = Response<Body>;

            fn from_request(
                req: &Request<()>,
                payload: &mut Payload,
            ) -> Result<Self, Self::Rejection> {
                $(
                    let $T = match <$T as crate::from_request::FromRequest>::from_request(&req, payload) {
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
