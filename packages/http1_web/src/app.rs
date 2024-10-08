use std::collections::HashMap;

use http1::{
    body::Body, handler::RequestHandler, method::Method, request::Request, response::Response,
    status::StatusCode,
};

use crate::{
    from_request::FromRequest,
    handler::{BoxedHandler, Handler},
    into_response::IntoResponse,
    middleware::BoxedMiddleware,
    router::Router,
    state::State,
};

pub struct App<T> {
    scope: Scope,
    fallback: Option<BoxedHandler>,
    middleware: Vec<BoxedMiddleware>,
    state: State<T>,
}

impl App<()> {
    pub fn new() -> App<()> {
        Self::with_state(())
    }

    pub fn with_state<T>(state: T) -> App<T> {
        App {
            scope: Scope::new(),
            fallback: None,
            middleware: Vec::new(),
            state: State::new(state),
        }
    }
}

impl Default for App<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> App<T> {
    pub fn middleware<M>(mut self, handler: M) -> Self
    where
        M: Fn(Request<Body>, &BoxedHandler) -> Response<Body> + Send + Sync + 'static,
    {
        self.middleware.push(BoxedMiddleware::new(handler));
        self
    }

    pub fn route<H, Args, R>(mut self, method: Method, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.scope
            .add_route(method, route, BoxedHandler::new(handler));
        self
    }

    pub fn get<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::GET, route, handler)
    }

    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::POST, route, handler)
    }

    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PUT, route, handler)
    }

    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::DELETE, route, handler)
    }

    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PATCH, route, handler)
    }

    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::OPTIONS, route, handler)
    }

    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::HEAD, route, handler)
    }

    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::TRACE, route, handler)
    }
}

impl<T> RequestHandler for App<T>
where
    T: 'static,
{
    fn handle(&self, mut req: Request<Body>) -> Response<Body> {
        let middlewares = self.middleware.as_slice();

        let method = req.method().clone();
        let req_path = req.uri().path_and_query().path().to_owned();
        let route_match = self
            .scope
            .method_router
            .get(&method)
            .and_then(|router| router.find(&req_path));

        match route_match {
            Some(mtch) => {
                // Add any additional extensions
                req.extensions_mut().insert(mtch.params.clone());
                req.extensions_mut().insert(self.state.clone());

                // Handle the request
                let res = if middlewares.is_empty() {
                    mtch.value.call(req)
                } else {
                    let handler = middlewares
                        .iter()
                        .cloned()
                        .fold(mtch.value.clone(), |cur, next| {
                            BoxedHandler::new(move |r| next.on_request(r, &cur))
                        });

                    handler.call(req)
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

pub struct Scope {
    method_router: HashMap<Method, Router<BoxedHandler>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            method_router: HashMap::new(),
        }
    }
}

impl Scope {
    pub fn scope(mut self, route: &str, scope: Scope) -> Self {
        for (method, router) in scope.method_router {
            for (r, handler) in router.into_entries() {
                let full_path = format!("{route}{r}");
                self.add_route(method.clone(), &full_path, handler);
            }
        }
        self
    }

    fn add_route(&mut self, method: Method, route: &str, handler: BoxedHandler) {
        match self.method_router.entry(method) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().insert(route, handler);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut router = Router::new();
                router.insert(route, handler);
                entry.insert(router);
            }
        }
    }

    pub fn route<H, Args, R>(mut self, method: Method, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.add_route(method, route, BoxedHandler::new(handler));
        self
    }

    pub fn get<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::GET, route, handler)
    }

    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::POST, route, handler)
    }

    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PUT, route, handler)
    }

    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::DELETE, route, handler)
    }

    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::PATCH, route, handler)
    }

    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::OPTIONS, route, handler)
    }

    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::HEAD, route, handler)
    }

    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(Method::TRACE, route, handler)
    }
}
