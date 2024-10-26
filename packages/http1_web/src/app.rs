use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::AtomicUsize, Arc, LazyLock},
};

use http1::{
    body::Body, handler::RequestHandler, method::Method, request::Request, response::Response,
    status::StatusCode,
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
            .add_route(route, method, BoxedHandler::new(handler));
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

static NOT_FOUND_HANDLER: LazyLock<BoxedHandler> =
    LazyLock::new(|| BoxedHandler::new(NotFoundHandler));

impl RequestHandler for App {
    fn handle(&self, mut req: Request<Body>) -> Response<Body> {
        let middlewares = self.middleware.as_slice();
        let method = MethodRoute::from_method(req.method());
        let req_path = req.uri().path_and_query().path();
        let mtch = self.scope.find(&req_path, req.method());

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RouteId(usize);
impl RouteId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let next = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        RouteId(next)
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    method_router: Router<RouteId>,
    path_to_route: HashMap<String, RouteId>,
    route_to_methods: HashMap<RouteId, HashMap<Method, BoxedHandler>>,
    fallbacks: Router<BoxedHandler>,
}

impl Scope {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_route(&mut self, route: &str, method: MethodRoute, handler: BoxedHandler) {
        log::debug!("Adding route: {method} => {route}");

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
                route.to_owned()
            } else {
                format!("{route}{sub_route}")
            };

            self.fallbacks.insert(full_path, fallback);
        }

        for (r, route_id) in scope.method_router.into_entries() {
            let sub_route = r.to_string();
            let full_path = if route == "/" {
                route.to_owned()
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

    fn find(&self, route: &str, method: &Method) -> Match<&BoxedHandler> {
        let fallback = self
            .fallbacks
            .find(route)
            .as_ref()
            .map(|m| m.value)
            .unwrap_or_else(|| &*NOT_FOUND_HANDLER);

        match self.method_router.find(route) {
            Some(Match {
                params,
                value: route_id,
            }) => {
                // let value = mtch.value.get(method).unwrap_or(fallback);

                // Match {
                //     params: mtch.params,
                //     value,
                // }

                let methods = self
                    .route_to_methods
                    .get(route_id)
                    .expect("route id it's define to methods are");

                let value = methods.get(method).unwrap_or(fallback);

                Match { params, value }
            }
            None => Match {
                params: ParamsMap::default(),
                value: &fallback,
            },
        }
    }

    pub fn fallback<H, Args, R>(mut self, fallback: H) -> Self
    where
        Args: FromRequest,
        H: Handler<Args, Output = R> + Sync + Send + 'static,
        R: IntoResponse,
    {
        self.add_fallback(BoxedHandler::new(fallback));
        self
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
        self.add_route(route, method, BoxedHandler::new(handler));
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

#[cfg(test)]
mod tests {
    use http1::body::http_body::HttpBody;

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

        dbg!(&entries);

        // Assert existing routes
        assert!(entries.iter().any(|x| x == "/hello"));
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
}
