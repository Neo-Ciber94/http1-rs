use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, LazyLock},
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

#[derive(Debug)]
struct MethodRouter {
    methods: HashMap<Method, BoxedHandler>,
    fallback: Option<BoxedHandler>,
}

#[derive(Debug)]
pub struct Scope(Router<MethodRouter>);

impl Scope {
    pub fn new() -> Self {
        Scope(Default::default())
    }

    fn add_route(&mut self, route: &str, method: MethodRoute, handler: BoxedHandler) {
        log::debug!("Adding route: {method} => {route}");

        match self.0.find_mut(route) {
            Some(mtch) => {
                for m in method.into_methods() {
                    if mtch.value.methods.insert(m, handler.clone()).is_some() {
                        panic!("handler already exists on {method}: {route}");
                    }
                }
            }
            None => {
                let mut methods = HashMap::new();
                for m in method.into_methods() {
                    methods.insert(m, handler.clone());
                }

                self.0.insert(
                    route,
                    MethodRouter {
                        methods,
                        fallback: None,
                    },
                );
            }
        }
    }

    fn add_fallback(&mut self, fallback: BoxedHandler) {
        match self.0.find_mut("/*") {
            Some(mtch) => mtch.value.fallback = Some(fallback),
            None => {
                self.0.insert(
                    "/*",
                    MethodRouter {
                        methods: HashMap::new(),
                        fallback: Some(fallback),
                    },
                );
            }
        }
    }

    fn add_scope(&mut self, route: &str, scope: Scope) {
        for (r, router) in scope.0.into_entries() {
            let sub_route = r.to_string();
            let full_path = if route == "/" {
                sub_route
            } else {
                format!("{route}{sub_route}")
            };

            if let Some(fallback) = router.fallback {
                self.add_fallback(fallback);
            }

            for (method, handler) in router.methods {
                let method_route = MethodRoute::from_method(&method);
                self.add_route(&full_path, method_route, handler);
            }
        }
    }

    fn find(&self, route: &str, method: &Method) -> Match<&BoxedHandler> {
        match self.0.find(route) {
            Some(mtch) => {
                let fallback = mtch
                    .value
                    .fallback
                    .as_ref()
                    .unwrap_or_else(|| &*NOT_FOUND_HANDLER);

                dbg!(route, &mtch, method);
                let handler = mtch.value.methods.get(method).unwrap_or(fallback);

                Match {
                    params: mtch.params,
                    value: handler,
                }
            }
            None => Match {
                params: ParamsMap::default(),
                value: &NOT_FOUND_HANDLER,
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

// impl Debug for Scope {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // let mut map = f.debug_map();

//         // for (method, handler) in self.0.method_router.iter() {
//         //     map.entry(&method, &handler);
//         // }

//         // map.finish()
//         f.debug_struct("Scope").finish_non_exhaustive()
//     }
// }

#[cfg(test)]
mod tests {
    use http1::body::http_body::HttpBody;

    use super::*;

    fn test_handler(response: &'static str) -> &'static str {
        response
    }

    fn get_response(handler: &BoxedHandler, method: Method) -> String {
        let req = Request::builder()
            .method(method)
            .body(Body::empty())
            .unwrap();
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
            .get("/test", || test_handler("get"))
            .post("/test", || test_handler("post"))
            .put("/test", || test_handler("put"))
            .delete("/test", || test_handler("delete"))
            .patch("/test", || test_handler("patch"))
            .options("/test", || test_handler("options"))
            .head("/test", || test_handler("head"))
            .trace("/test", || test_handler("trace"));

        // Test each method
        assert_eq!(
            get_response(scope.find("/test", &Method::GET).value, Method::GET),
            "get"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::POST).value, Method::POST),
            "post"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PUT).value, Method::PUT),
            "put"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::DELETE).value, Method::DELETE),
            "delete"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PATCH).value, Method::PATCH),
            "patch"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::OPTIONS).value, Method::OPTIONS),
            "options"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::HEAD).value, Method::HEAD),
            "head"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::TRACE).value, Method::TRACE),
            "trace"
        );
    }

    #[test]
    fn test_method_precedence() {
        let scope = Scope::new()
            .get("/api", || test_handler("get_handler"))
            .post("/api", || test_handler("post_handler"));

        // Specific methods should take precedence over any()
        assert_eq!(
            get_response(scope.find("/api", &Method::GET).value, Method::GET),
            "get_handler"
        );
        assert_eq!(
            get_response(scope.find("/api", &Method::POST).value, Method::POST),
            "post_handler"
        );
    }

    #[test]
    fn test_nested_routes() {
        let nested = Scope::new()
            .get("/users", || test_handler("get_users"))
            .post("/users", || test_handler("create_user"))
            .put("/users", || test_handler("update_user"))
            .delete("/users", || test_handler("delete_user"));

        let scope = Scope::new()
            .scope("/api", nested)
            .get("/health", || test_handler("health"))
            .any("/*", || test_handler("fallback"));

        assert_eq!(
            get_response(scope.find("/api/users", &Method::GET).value, Method::GET),
            "get_users"
        );
        assert_eq!(
            get_response(scope.find("/api/users", &Method::POST).value, Method::POST),
            "create_user"
        );
        assert_eq!(
            get_response(scope.find("/api/users", &Method::PUT).value, Method::PUT),
            "update_user"
        );
        assert_eq!(
            get_response(
                scope.find("/api/users", &Method::DELETE).value,
                Method::DELETE
            ),
            "delete_user"
        );
        assert_eq!(
            get_response(scope.find("/health", &Method::GET).value, Method::GET),
            "health"
        );
        assert_eq!(
            get_response(scope.find("/not-found", &Method::GET).value, Method::GET),
            "fallback"
        );
    }

    #[test]
    fn test_any_method() {
        let scope = Scope::new().any("/test", || test_handler("any_handler"));

        // Test all methods with the any() route
        assert_eq!(
            get_response(scope.find("/test", &Method::GET).value, Method::GET),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::POST).value, Method::POST),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PUT).value, Method::PUT),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::DELETE).value, Method::DELETE),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::PATCH).value, Method::PATCH),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::OPTIONS).value, Method::OPTIONS),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::HEAD).value, Method::HEAD),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::TRACE).value, Method::TRACE),
            "any_handler"
        );
        assert_eq!(
            get_response(scope.find("/test", &Method::CONNECT).value, Method::CONNECT),
            "any_handler"
        );
    }

    #[test]
    fn test_wildcard_routes() {
        let scope = Scope::new()
            .get("/api/users/*", || test_handler("users_wildcard"))
            .post("/api/users/create/:path*", || {
                test_handler("create_wildcard")
            });

        assert_eq!(
            get_response(
                scope.find("/api/users/123", &Method::GET).value,
                Method::GET
            ),
            "users_wildcard"
        );

        assert_eq!(
            get_response(
                scope.find("/api/users/123/profile", &Method::GET).value,
                Method::GET
            ),
            "users_wildcard"
        );

        assert_eq!(
            get_response(
                scope.find("/api/users/create", &Method::POST).value,
                Method::POST
            ),
            "create_wildcard"
        );
    }

    #[test]
    fn test_combined_method_routes() {
        let scope = Scope::new().route(MethodRoute::GET | MethodRoute::POST, "/combined", || {
            test_handler("combined_handler")
        });

        // Test GET and POST combined method
        assert_eq!(
            get_response(scope.find("/combined", &Method::GET).value, Method::GET),
            "combined_handler"
        );
        assert_eq!(
            get_response(scope.find("/combined", &Method::POST).value, Method::POST),
            "combined_handler"
        );
    }

    #[test]
    fn test_fallback_handler() {
        let scope = Scope::new()
            .fallback(|| test_handler("fallback_handler")) // Set fallback handler
            .get("/test", || test_handler("get_handler"));

        let get_match = scope.find("/test", &Method::GET);
        let not_found_match = scope.find("/not-found", &Method::GET);

        assert_eq!(get_response(get_match.value, Method::GET), "get_handler");
        assert_eq!(
            get_response(not_found_match.value, Method::GET),
            "fallback_handler"
        );
    }

    #[test]
    fn test_fallback_with_nested_scope() {
        let nested_scope = Scope::new()
            .fallback(|| test_handler("nested_fallback"))
            .get("/inner", || test_handler("inner_handler"));

        let scope = Scope::new().scope("/outer", nested_scope);

        let inner_match = scope.find("/outer/inner", &Method::GET);
        let not_found_match = scope.find("/outer/not-found", &Method::GET);

        assert_eq!(
            get_response(inner_match.value, Method::GET),
            "inner_handler"
        );
        assert_eq!(
            get_response(not_found_match.value, Method::GET),
            "nested_fallback"
        );
    }

    #[test]
    fn test_multiple_fallbacks() {
        let scope = Scope::new()
            .fallback(|| test_handler("global_fallback"))
            .scope(
                "/api",
                Scope::new()
                    .fallback(|| test_handler("api_fallback"))
                    .get("/test", || test_handler("api_handler")),
            );

        let api_match = scope.find("/api/test", &Method::GET);
        let global_not_found_match = scope.find("/not-found", &Method::GET);
        let api_not_found_match = scope.find("/api/not-found", &Method::GET);

        assert_eq!(get_response(api_match.value, Method::GET), "api_handler");
        assert_eq!(
            get_response(global_not_found_match.value, Method::GET),
            "global_fallback"
        );
        assert_eq!(
            get_response(api_not_found_match.value, Method::GET),
            "api_fallback"
        );
    }
}
