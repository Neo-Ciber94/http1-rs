mod pre_render;
pub use pre_render::*;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::AtomicUsize, LazyLock},
};

use http1::{
    body::Body, extensions::Extensions, handler::RequestHandler, method::Method, request::Request,
    response::Response, status::StatusCode,
};

use crate::{
    from_request::FromRequest,
    handler::{BoxedHandler, Handler},
    middleware::{BoxedMiddleware, Middleware},
    routing::{
        method_route::MethodRoute, params::ParamsMap, route::Route, route_info::RouteInfo, Match,
        Router,
    },
    state::State,
    IntoResponse,
};

/// Represents a server application that provides middlewares and handlers.
#[derive(Debug)]
pub struct App {
    scope: Scope,
    middleware: Vec<BoxedMiddleware>,
    app_state: Extensions,
}

impl App {
    /// Constructs an empty server application.
    pub fn new() -> App {
        App {
            scope: Scope::root(),
            middleware: Vec::new(),
            app_state: Default::default(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Adds a middleware to this application.
    pub fn add_middleware<M>(&mut self, middleware: M)
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.middleware.push(BoxedMiddleware::new(middleware));
    }

    /// Adds a route handler to this application.
    pub fn add_route<H, Args, R>(&mut self, method: MethodRoute, route: &str, handler: H)
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.scope
            .add_route(route, method, BoxedHandler::new(handler));
    }
}

impl App {
    /// Adds a middleware.
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.add_middleware(middleware);
        self
    }

    /// Adds a state to the app, this later can be retrieve using `State<T>`.
    pub fn state<U>(mut self, value: U) -> Self
    where
        U: Clone + Send + Sync + 'static,
    {
        self.app_state.insert(State::new(value));
        self
    }

    /// Adds a nested route.
    pub fn scope(mut self, route: &str, scope: Scope) -> Self {
        self.scope.add_scope(route, scope);
        self
    }

    /// Adds a route with the given method, route path and handler.
    pub fn route<H, Args, R>(mut self, method: MethodRoute, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.add_route(method, route, handler);
        self
    }

    /// Adds a fallback route handler.
    pub fn fallback<H, Args, R>(mut self, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.scope.add_fallback(BoxedHandler::new(handler));
        self
    }

    /// Adds a `GET` request route handler.
    pub fn get<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::GET, route, handler)
    }

    /// Adds a `POST` request route handler.
    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::POST, route, handler)
    }

    /// Adds a `PUT` request route handler.
    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PUT, route, handler)
    }

    /// Adds a `DELETE` request route handler.
    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::DELETE, route, handler)
    }

    /// Adds a `PATCH` request route handler.
    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PATCH, route, handler)
    }

    /// Adds a `OPTIONS` request route handler.
    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::OPTIONS, route, handler)
    }

    /// Adds a `HEAD` request route handler.
    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::HEAD, route, handler)
    }

    /// Adds a `TRACE` request route handler.
    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::TRACE, route, handler)
    }

    /// Adds a handler that catches any route in the given route path.
    pub fn any<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::any(), route, handler)
    }

    /// Return an iterator over all the routes.
    pub fn routes(&self) -> impl Iterator<Item = (Route, &Method)> {
        self.scope.routes()
    }
}

static NOT_FOUND_HANDLER: LazyLock<BoxedHandler> =
    LazyLock::new(|| BoxedHandler::new(DefaultFallback));

impl RequestHandler for App {
    fn handle(&self, mut req: Request<Body>) -> Response<Body> {
        let middlewares = self.middleware.as_slice();
        let method = MethodRoute::from_method(req.method());
        let req_path = req.uri().path_and_query().path();
        let mtch = self.scope.find(req_path, req.method());

        // Add any additional extensions
        if let Some(r) = self.scope.find_route(req_path) {
            req.extensions_mut().insert(RouteInfo(r));
        }

        req.extensions_mut().insert(mtch.params.clone());
        req.extensions_mut().extend(self.app_state.clone());

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

struct DefaultFallback;

impl Handler<Request<Body>> for DefaultFallback {
    type Output = Response<Body>;

    fn call(&self, _: Request<Body>) -> Self::Output {
        Response::new(StatusCode::NOT_FOUND, Body::empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RouteId(usize);
impl RouteId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let next = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        RouteId(next)
    }
}

/// An scoped route.
#[derive(Default)]
pub struct Scope {
    method_router: Router<RouteId>,
    path_to_route: HashMap<String, RouteId>,
    route_to_methods: HashMap<RouteId, HashMap<Method, BoxedHandler>>,
    fallbacks: Router<BoxedHandler>,
    is_root: bool,
}

impl Scope {
    pub(crate) fn root() -> Self {
        Scope {
            is_root: true,
            ..Default::default()
        }
    }

    /// Creates a new scoped route.
    pub fn new() -> Self {
        Default::default()
    }

    fn add_route(&mut self, route: &str, method: MethodRoute, handler: BoxedHandler) {
        if self.is_root {
            log::debug!("Adding route: {method} => {route}");
        }

        match self.path_to_route.get(route) {
            Some(route_id) => {
                let methods = self
                    .route_to_methods
                    .get_mut(route_id)
                    .expect("no handler route");

                for m in method.into_methods() {
                    methods.insert(m, handler.clone());
                }
            }
            None => {
                let mut methods = HashMap::new();
                for m in method.into_methods() {
                    methods.insert(m, handler.clone());
                }

                let route_id = RouteId::next();
                self.method_router.insert(route, route_id);
                self.path_to_route.insert(route.to_owned(), route_id);
                self.route_to_methods.insert(route_id, methods);
            }
        }
    }

    fn add_fallback(&mut self, fallback: BoxedHandler) {
        self.fallbacks.insert("/", fallback.clone());
        self.fallbacks.insert("/*", fallback);
    }

    fn add_scope(&mut self, route: &str, mut scope: Scope) {
        // First add the fallbacks
        for (r, fallback) in scope.fallbacks.into_entries() {
            let sub_route = r.to_string();
            let full_path = if route == "/" {
                sub_route
            } else {
                format!("{route}{sub_route}")
            };

            self.fallbacks.insert(full_path, fallback);
        }

        for (r, route_id) in scope.method_router.into_entries() {
            let sub_route = r.to_string();
            let full_path = if route == "/" {
                sub_route
            } else {
                format!("{route}{sub_route}")
            };

            let methods = scope.route_to_methods.remove(&route_id).expect("no routes");
            for (m, handler) in methods {
                let method_route = MethodRoute::from_method(&m);
                self.add_route(&full_path, method_route, handler);
            }
        }
    }

    fn find_fallback(&self, route: &str) -> &BoxedHandler {
        self.fallbacks
            .find(route)
            .as_ref()
            .map(|m| m.value)
            .unwrap_or_else(|| &*NOT_FOUND_HANDLER)
    }

    fn find(&self, route: &str, method: &Method) -> Match<&BoxedHandler> {
        let fallback = self.find_fallback(route);

        match self.method_router.find(route) {
            Some(Match {
                params,
                value: route_id,
            }) => {
                let methods = self
                    .route_to_methods
                    .get(route_id)
                    .expect("route id it's define to methods are");

                let value = methods.get(method).unwrap_or(fallback);

                Match { params, value }
            }
            None => Match {
                params: ParamsMap::default(),
                value: fallback,
            },
        }
    }

    fn find_route(&self, route: &str) -> Option<Route> {
        match self.method_router.find(route) {
            Some(Match { value, .. }) => self.method_router.entries().find_map(|(route, id)| {
                if id == value {
                    Some(route.clone())
                } else {
                    None
                }
            }),
            None => {
                // Try to find the fallback
                let fallback = self.find_fallback(route);
                self.fallbacks.entries().find_map(|(route, handler)| {
                    if handler == fallback {
                        Some(route.clone())
                    } else {
                        None
                    }
                })
            }
        }
    }

    /// Adds a fallback route handler.
    pub fn fallback<H, Args, R>(mut self, fallback: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.add_fallback(BoxedHandler::new(fallback));
        self
    }

    /// Adds a nested route.
    pub fn scope(mut self, route: &str, scope: Scope) -> Self {
        self.add_scope(route, scope);
        self
    }

    /// Adds a route with the given method, route path and handler.
    pub fn route<H, Args, R>(mut self, method: MethodRoute, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.add_route(route, method, BoxedHandler::new(handler));
        self
    }

    /// Adds a `GET` request route handler.
    pub fn get<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::GET, route, handler)
    }

    /// Adds a `POST` request route handler.
    pub fn post<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::POST, route, handler)
    }

    /// Adds a `PUT` request route handler.
    pub fn put<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PUT, route, handler)
    }

    /// Adds a `DELETE` request route handler.
    pub fn delete<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::DELETE, route, handler)
    }

    /// Adds a `PATCH` request route handler.
    pub fn patch<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::PATCH, route, handler)
    }

    /// Adds a `OPTIONS` request route handler.
    pub fn options<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::OPTIONS, route, handler)
    }

    /// Adds a `HEAD` request route handler.
    pub fn head<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::HEAD, route, handler)
    }

    /// Adds a `TRACE` request route handler.
    pub fn trace<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::TRACE, route, handler)
    }

    /// Adds a handler that catches any route in the given route path.
    pub fn any<H, Args, R>(self, route: &str, handler: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.route(MethodRoute::any(), route, handler)
    }

    /// Return an iterator over all the routes.
    pub fn routes(&self) -> impl Iterator<Item = (Route, &Method)> {
        self.method_router.entries().flat_map(|(r, id)| {
            self.route_to_methods
                .get(id)
                .expect("failed to get route methods")
                .keys()
                .map(|method| (r.clone(), method))
        })
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for (route, route_id) in self.method_router.entries() {
            let methods = self.route_to_methods.get(route_id).expect("no methods");
            map.entry(route, methods);
        }

        map.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use http1::{body::http_body::HttpBody, uri::uri::Uri};

    use crate::state::State;

    use super::*;

    fn get_response(handler: &BoxedHandler) -> String {
        let req = Request::builder().body(Body::empty()).unwrap();
        let res = handler.call(req);
        let bytes = res
            .into_body()
            .read_all_bytes()
            .expect("failed to read bytes");
        String::from_utf8(bytes).expect("failed to read bytes as utf-8")
    }

    #[test]
    fn test_all_http_methods() {
        let scope = Scope::new()
            .get("/test", || "get")
            .post("/test", || "post")
            .put("/test", || "put")
            .delete("/test", || "delete")
            .patch("/test", || "patch")
            .options("/test", || "options")
            .head("/test", || "head")
            .trace("/test", || "trace");

        // Test each method
        assert_eq!(get_response(scope.find("/test", &Method::GET).value), "get");
        assert_eq!(
            get_response(scope.find("/test", &Method::POST).value),
            "post"
        );
        assert_eq!(get_response(scope.find("/test", &Method::PUT).value), "put");
        assert_eq!(
            get_response(scope.find("/test", &Method::DELETE).value),
            "delete"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PATCH).value),
            "patch"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::OPTIONS).value),
            "options"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::HEAD).value),
            "head"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::TRACE).value),
            "trace"
        );
    }

    #[test]
    fn test_method_precedence() {
        let scope = Scope::new()
            .get("/api", || "get_handler")
            .post("/api", || "post_handler");

        // Specific methods should take precedence over any()
        assert_eq!(
            get_response(scope.find("/api", &Method::GET).value),
            "get_handler"
        );
        assert_eq!(
            get_response(scope.find("/api", &Method::POST).value),
            "post_handler"
        );
    }

    #[test]
    fn test_nested_routes() {
        let nested = Scope::new()
            .get("/users", || "get_users")
            .post("/users", || "create_user")
            .put("/users", || "update_user")
            .delete("/users", || "delete_user");

        let scope = Scope::new()
            .scope("/api", nested)
            .get("/health", || "health")
            .any("/*", || "fallback");

        assert_eq!(
            get_response(scope.find("/api/users", &Method::GET).value),
            "get_users"
        );
        assert_eq!(
            get_response(scope.find("/api/users", &Method::POST).value),
            "create_user"
        );
        assert_eq!(
            get_response(scope.find("/api/users", &Method::PUT).value),
            "update_user"
        );
        assert_eq!(
            get_response(scope.find("/api/users", &Method::DELETE).value),
            "delete_user"
        );
        assert_eq!(
            get_response(scope.find("/health", &Method::GET).value),
            "health"
        );
        assert_eq!(
            get_response(scope.find("/not_found", &Method::POST).value),
            "fallback"
        );
    }

    #[test]
    fn test_any_method() {
        let scope = Scope::new().any("/test", || "any_handler");

        // Test all methods with the any() route
        assert_eq!(
            get_response(scope.find("/test", &Method::GET).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::POST).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PUT).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::DELETE).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PATCH).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::OPTIONS).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::HEAD).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::TRACE).value),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::CONNECT).value),
            "any_handler"
        );
    }

    #[test]
    fn test_wildcard_routes() {
        let scope = Scope::new()
            .get("/api/users/*", || "users_wildcard")
            .post("/api/users/create/:path*", || "create_wildcard");

        assert_eq!(
            get_response(scope.find("/api/users/123", &Method::GET).value),
            "users_wildcard"
        );
        assert_eq!(
            get_response(scope.find("/api/users/123/profile", &Method::GET).value),
            "users_wildcard"
        );
        assert_eq!(
            get_response(scope.find("/api/users/create", &Method::POST).value),
            "create_wildcard"
        );
    }

    #[test]
    fn test_combined_method_routes() {
        let scope = Scope::new().route(MethodRoute::GET | MethodRoute::POST, "/combined", || {
            "combined_handler"
        });

        // Test GET and POST combined method
        assert_eq!(
            get_response(scope.find("/combined", &Method::GET).value),
            "combined_handler"
        );
        assert_eq!(
            get_response(scope.find("/combined", &Method::POST).value),
            "combined_handler"
        );
    }

    #[test]
    fn test_fallback_handler() {
        let scope = Scope::new()
            .get("/test", || "get_handler")
            .fallback(|| "fallback_handler"); // Set fallback handler

        let get_match = scope.find("/test", &Method::GET);
        let not_found_match = scope.find("/not-found", &Method::GET);

        assert_eq!(get_response(get_match.value), "get_handler");
        assert_eq!(get_response(not_found_match.value), "fallback_handler");
    }

    #[test]
    fn test_fallback_with_nested_scope() {
        let nested_scope = Scope::new()
            .get("/inner", || "inner_handler")
            .fallback(|| "nested_fallback");

        let scope = Scope::new().scope("/outer", nested_scope);

        let inner_match = scope.find("/outer/inner", &Method::GET);
        let not_found_match = scope.find("/outer/not-found", &Method::GET);

        assert_eq!(get_response(inner_match.value), "inner_handler");
        assert_eq!(get_response(not_found_match.value), "nested_fallback");
    }

    #[test]
    fn test_multiple_fallbacks() {
        let scope = Scope::new()
            .scope(
                "/api",
                Scope::new()
                    .get("/test", || "api_handler")
                    .fallback(|| "api_fallback"),
            )
            .fallback(|| "global_fallback");

        let api_match = scope.find("/api/test", &Method::GET);
        let global_not_found_match = scope.find("/not-found", &Method::GET);
        let api_not_found_match = scope.find("/api/not-found", &Method::GET);

        assert_eq!(get_response(api_match.value), "api_handler");
        assert_eq!(
            get_response(global_not_found_match.value),
            "global_fallback"
        );
        assert_eq!(get_response(api_not_found_match.value), "api_fallback");
    }

    #[test]
    fn test_add_fallback_first() {
        let scope = Scope::new()
            .fallback(|| "fallback")
            .post("/other", || "other");

        let other_match = scope.find("/other", &Method::POST);
        let not_found_match = scope.find("/not-found", &Method::PUT);

        assert_eq!(get_response(other_match.value), "other");
        assert_eq!(get_response(not_found_match.value), "fallback");
    }

    #[test]
    fn test_add_fallback_first_nested() {
        let scope = Scope::new()
            .fallback(|| "fallback")
            .scope(
                "/api",
                Scope::new()
                    .fallback(|| "nested_fallback")
                    .delete("/items", || "items"),
            )
            .post("/other", || "other");

        let other_match = scope.find("/other", &Method::POST);
        let nested_api_match = scope.find("/api/items", &Method::DELETE);
        let not_found_match = scope.find("/not-found", &Method::PUT);
        let nested_fallback = scope.find("/api/not-existing", &Method::PUT);

        assert_eq!(get_response(other_match.value), "other");
        assert_eq!(get_response(not_found_match.value), "fallback");
        assert_eq!(get_response(nested_api_match.value), "items");
        assert_eq!(get_response(nested_fallback.value), "nested_fallback");
    }

    #[test]
    fn should_use_parent_fallback_in_nested() {
        let scope = Scope::new()
            .fallback(|| "root fallback")
            .scope("/api", Scope::new().post("/items", || "items"));

        let not_found_match = scope.find("/not-found", &Method::POST);
        let items_match = scope.find("/api/items", &Method::POST);
        let not_found_items_match = scope.find("/api/items", &Method::GET);
        let parent_fallback_match = scope.find("/api/not-found", &Method::POST);

        assert_eq!(get_response(not_found_match.value), "root fallback");
        assert_eq!(get_response(items_match.value), "items");
        assert_eq!(get_response(not_found_items_match.value), "root fallback");
        assert_eq!(get_response(parent_fallback_match.value), "root fallback");
    }

    #[test]
    fn should_list_routes() {
        let scope = Scope::new()
            .get("/hello", || "hello")
            .get("/*", || "static files")
            .scope("/", Scope::new().get("/health", || "ok"))
            .scope(
                "/api",
                Scope::new()
                    .get("/users", || "user_list")
                    .post("/users", || "create_user")
                    .scope(
                        "/admin",
                        Scope::new()
                            .get("/settings", || "admin_settings")
                            .post("/settings", || "update_settings")
                            .get("/settings/:id", || "get_setting"),
                    )
                    .scope(
                        "/posts",
                        Scope::new()
                            .get("/", || "list_posts")
                            .get("/new", || "new_post")
                            .post("/", || "create_post")
                            .get("/*", || "catch_all_posts") // Unnamed catch-all
                            .get("/featured/:post*", || "featured_posts"), // Named catch-all
                    )
                    .get("/items/:item_id", || "get_item") // Parameterized route
                    .get("/items/:item_id/details/*", || "item_details"), // Parameterized route with catch-all
            );

        let entries = scope
            .method_router
            .into_entries()
            .map(|(r, _)| r.to_string())
            .collect::<Vec<_>>();

        // Assert existing routes
        assert!(entries.iter().any(|x| x == "/hello"));
        assert!(entries.iter().any(|x| x == "/health"));
        assert!(entries.iter().any(|x| x == "/*")); // root catch-all
        assert!(entries.iter().any(|x| x == "/api/users"));
        assert!(entries.iter().any(|x| x == "/api/admin/settings"));
        assert!(entries.iter().any(|x| x == "/api/admin/settings/:id")); // Parameterized
        assert!(entries.iter().any(|x| x == "/api/posts"));
        assert!(entries.iter().any(|x| x == "/api/posts/new"));
        assert!(entries.iter().any(|x| x == "/api/posts/*")); // Unnamed catch-all
        assert!(entries.iter().any(|x| x == "/api/posts/featured/:post*")); // Named catch-all
        assert!(entries.iter().any(|x| x == "/api/items/:item_id")); // Parameterized route
        assert!(entries.iter().any(|x| x == "/api/items/:item_id/details/*")); // Parameterized route with catch-all
    }

    #[test]
    fn should_get_state() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct HitPoints(u32);

        let s = Arc::new(Mutex::new(None));

        let app = {
            let s = Arc::clone(&s);

            App::new()
                .state(HitPoints(10))
                .get("/", move |state: State<HitPoints>| {
                    let mut lock = s.lock().unwrap();
                    *lock = Some(state);
                })
        };

        app.handle(Request::new(
            Method::GET,
            Uri::from_str("/").unwrap(),
            Body::empty(),
        ));

        let state = s.lock().unwrap().take().unwrap();
        assert_eq!(state.0, HitPoints(10))
    }
}
