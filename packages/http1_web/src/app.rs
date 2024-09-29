use std::{collections::HashMap, sync::Arc};

use http1::{
    body::Body, handler::RequestHandler, method::Method, request::Request, response::Response,
    status::StatusCode,
};

use crate::{
    from_request::{FromRequest, FromRequestRef},
    into_response::IntoResponse,
    router::Router,
};

pub trait Handler<Args> {
    type Output: IntoResponse;
    fn call(&self, args: Args) -> Self::Output;
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

#[derive(Clone)]
struct BoxedHandler(Arc<dyn Fn(Request<Body>) -> Response<Body> + Sync + Send + 'static>);

impl BoxedHandler {
    pub fn new<H, Args, R>(handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        BoxedHandler(Arc::new(move |req| match Args::from_request(req) {
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

pub struct App<'a> {
    method_router: HashMap<Method, Router<'a, BoxedHandler>>,
    fallback: Option<BoxedHandler>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            method_router: HashMap::with_capacity(4),
            fallback: None,
        }
    }

    pub fn route<H, Args, R>(mut self, method: Method, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        match self.method_router.entry(method) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().insert(route, BoxedHandler::new(handler));
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut router = Router::new();
                router.insert(route, BoxedHandler::new(handler));
                entry.insert(router);
            }
        }

        self
    }

    pub fn get<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::GET, route, handler)
    }

    pub fn post<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::POST, route, handler)
    }

    pub fn put<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PUT, route, handler)
    }

    pub fn delete<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::DELETE, route, handler)
    }

    pub fn patch<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PATCH, route, handler)
    }

    pub fn options<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::OPTIONS, route, handler)
    }

    pub fn head<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::HEAD, route, handler)
    }

    pub fn trace<H, Args, R>(self, route: &'a str, handler: H) -> Self
    where
        Args: FromRequestRef,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::TRACE, route, handler)
    }
}

impl RequestHandler for App<'_> {
    fn handle(&self, mut req: Request<Body>) -> Response<Body> {
        let method = req.method().clone();
        let req_path = req.uri().path_and_query().path().to_owned();
        let route_match = self
            .method_router
            .get(&method)
            .and_then(|router| router.find(&req_path));

        match route_match {
            Some(m) => {
                // Add the params as a extension
                req.extensions_mut().insert(m.params.clone());
                let res = m.value.call(req);

                match method {
                    // We don't need the body for HEAD requests
                    Method::HEAD => res.map_body(|_| Body::empty()),
                    _ => res,
                }
            }
            None => match &self.fallback {
                Some(fallback) => fallback.call(req),
                None => not_found_handler(req),
            },
        }
    }
}

fn not_found_handler(_: Request<Body>) -> Response<Body> {
    Response::new(StatusCode::NOT_FOUND, Body::empty())
}
