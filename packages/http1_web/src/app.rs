use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, LazyLock},
};

use http1::{
    body::Body, handler::RequestHandler, request::Request, response::Response, status::StatusCode,
};

use crate::{
    from_request::FromRequest,
    handler::{BoxedHandler, Handler},
    into_response::IntoResponse,
    middleware::{BoxedMiddleware, Middleware},
    routing::{method_route::MethodRoute, params::ParamsMap, Match, Router},
    state::AppState,
};

#[derive(Debug)]
pub struct App {
    scope: Scope,
    fallback: Option<BoxedHandler>,
    middleware: Vec<BoxedMiddleware>,
    app_state: Arc<AppState>,
}

impl App {
    pub fn new() -> App {
        App {
            scope: Scope::new(),
            fallback: None,
            middleware: Vec::new(),
            app_state: Arc::new(AppState::default()),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.middleware.push(BoxedMiddleware::new(middleware));
        self
    }

    pub fn state<U>(mut self, value: U) -> Self
    where
        U: Clone + Send + Sync + 'static,
    {
        Arc::get_mut(&mut self.app_state)
            .expect("Failed to get data map")
            .insert(value);
        self
    }

    pub fn scope(mut self, route: &str, scope: Scope) -> Self {
        self.scope.add_scope(route, scope);
        self
    }

    pub fn route<H, Args, R>(mut self, method: MethodRoute, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.scope
            .add_route(method, route, BoxedHandler::new(handler));
        self
    }

    pub fn fallback<H, Args, R>(mut self, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.fallback = Some(BoxedHandler::new(handler));
        self
    }

    pub fn get<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::GET, route, handler)
    }

    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::POST, route, handler)
    }

    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PUT, route, handler)
    }

    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::DELETE, route, handler)
    }

    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PATCH, route, handler)
    }

    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::OPTIONS, route, handler)
    }

    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::HEAD, route, handler)
    }

    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::TRACE, route, handler)
    }

    pub fn any<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::any(), route, handler)
    }
}

static NOT_FOUND_HANDLER: LazyLock<BoxedHandler> = LazyLock::new(|| BoxedHandler::new(NotFoundHandler));

impl RequestHandler for App {
    fn handle(&self, mut req: Request<Body>) -> Response<Body> {
        let middlewares = self.middleware.as_slice();

        let fallback = self
            .fallback
            .as_ref()
            .unwrap_or_else(|| &*NOT_FOUND_HANDLER);

        let method = MethodRoute::from_method(req.method());
        let req_path = req.uri().path_and_query().path().to_owned();
        let mtch = self
            .scope
            .method_router
            .get(&method)
            .and_then(|router| router.find(&req_path))
            .unwrap_or_else(|| Match {
                params: ParamsMap::default(),
                value: fallback,
            });

        // Add any additional extensions
        req.extensions_mut().insert(mtch.params.clone());
        req.extensions_mut().insert(self.app_state.clone());

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
            MethodRoute::HEAD => res.map_body(|_| Body::empty()),
            _ => res,
        }
    }
}

struct NotFoundHandler;

impl Handler<Request<Body>> for NotFoundHandler {
    type Output = Response<Body>;

    fn call(&self, _: Request<Body>) -> Self::Output {
        Response::new(StatusCode::NOT_FOUND, Body::empty())
    }
}

pub struct Scope {
    method_router: HashMap<MethodRoute, Router<BoxedHandler>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            method_router: HashMap::new(),
        }
    }

    fn add_scope(&mut self, route: &str, scope: Scope) {
        for (method, router) in scope.method_router {
            for (r, handler) in router.into_entries() {
                let sub_route = r.to_string();
                let full_path = if route == "/" {
                    sub_route
                } else {
                    format!("{route}{sub_route}")
                };
                self.add_route(method.clone(), &full_path, handler);
            }
        }
    }

    fn add_route(&mut self, method: MethodRoute, route: &str, handler: BoxedHandler) {
        log::debug!("Adding route: {method} => {route}");

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

    pub fn scope(mut self, route: &str, scope: Scope) -> Self {
        self.add_scope(route, scope);
        self
    }

    pub fn route<H, Args, R>(mut self, method: MethodRoute, route: &str, handler: H) -> Self
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
        self.route(MethodRoute::GET, route, handler)
    }

    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::POST, route, handler)
    }

    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PUT, route, handler)
    }

    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::DELETE, route, handler)
    }

    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PATCH, route, handler)
    }

    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::OPTIONS, route, handler)
    }

    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::HEAD, route, handler)
    }

    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::TRACE, route, handler)
    }

    pub fn any<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::any(), route, handler)
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for (method, handler) in self.method_router.iter() {
            map.entry(&method, &handler);
        }

        map.finish()
    }
}
