use std::collections::HashMap;

use http1::{
    body::Body, handler::RequestHandler, method::Method, request::Request, response::Response,
    status::StatusCode,
};

use crate::{
    from_request::FromRequestRef,
    handler::{BoxedHandler, Handler},
    into_response::IntoResponse,
    middleware::BoxedMiddleware,
    router::Router,
};

pub struct App<'a> {
    method_router: HashMap<Method, Router<'a, BoxedHandler>>,
    fallback: Option<BoxedHandler>,
    on_request: Option<BoxedMiddleware>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            method_router: HashMap::with_capacity(4),
            fallback: None,
            on_request: None,
        }
    }

    pub fn on_request<M>(mut self, handler: M) -> Self
    where
        M: Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static,
    {
        self.on_request = Some(BoxedMiddleware::new(handler));
        self
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
        let middleware = self.on_request.as_ref();

        let method = req.method().clone();
        let req_path = req.uri().path_and_query().path().to_owned();
        let route_match = self
            .method_router
            .get(&method)
            .and_then(|router| router.find(&req_path));

        match route_match {
            Some(mtch) => {
                // Add the params as a extension
                req.extensions_mut().insert(mtch.params.clone());

                let res = match middleware {
                    Some(middleware) => middleware.on_request(req, mtch.value),
                    None => mtch.value.call(req),
                };

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
