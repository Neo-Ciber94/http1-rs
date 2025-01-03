#![allow(clippy::type_complexity)]

use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use http1::{
    body::Body,
    headers::{self, HeaderName, HeaderValue, Headers},
    method::Method,
    request::Request,
    response::Response,
    status::StatusCode,
};

use super::Middleware;

pub enum CorsOrigin {
    Any,
    List(HashSet<String>),
    Dynamic(Box<dyn Fn(&Request<Body>) -> bool + Send + Sync>),
}

#[derive(Debug)]
pub enum CorsValue<T> {
    None,
    Any,
    List(HashSet<T>),
}

impl Debug for CorsOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorsOrigin::Any => f.debug_tuple("Any").finish(),
            CorsOrigin::List(x) => f.debug_tuple("List").field(x).finish(),
            CorsOrigin::Dynamic(_) => f.debug_tuple("Dynamic").finish(),
        }
    }
}

#[derive(Debug)]
pub struct Cors {
    allowed_origins: CorsOrigin,
    allowed_methods: CorsValue<Method>,
    allowed_headers: CorsValue<HeaderName>,
    allow_credentials: Option<bool>,
    expose_headers: CorsValue<HeaderName>,
    max_age: Option<u64>,
}

impl Cors {
    pub fn builder() -> CorsBuilder {
        CorsBuilder::new()
    }

    pub fn any() -> Self {
        CorsBuilder::new()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .build()
    }

    pub fn with_origins<I, T>(origins: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        CorsBuilder::new()
            .allow_origins(origins)
            .allow_any_method()
            .allow_any_header()
            .build()
    }

    pub fn from_origins_fn<F>(f: F) -> Self
    where
        F: Fn(&Request<Body>) -> bool + Send + Sync + 'static,
    {
        CorsBuilder::new()
            .with_origin(f)
            .allow_any_method()
            .allow_any_header()
            .build()
    }

    pub fn allowed_origins(&self) -> &CorsOrigin {
        &self.allowed_origins
    }

    pub fn allowed_methods(&self) -> &CorsValue<Method> {
        &self.allowed_methods
    }

    pub fn allowed_headers(&self) -> &CorsValue<HeaderName> {
        &self.allowed_headers
    }

    pub fn allow_credentials(&self) -> Option<bool> {
        self.allow_credentials
    }

    pub fn expose_headers(&self) -> &CorsValue<HeaderName> {
        &self.expose_headers
    }

    pub fn max_age(&self) -> Option<u64> {
        self.max_age
    }
}

pub struct CorsBuilder(Cors);

impl CorsBuilder {
    pub fn new() -> Self {
        CorsBuilder(Cors {
            allowed_origins: CorsOrigin::Any,
            allow_credentials: None,
            allowed_methods: CorsValue::None,
            allowed_headers: CorsValue::None,
            expose_headers: CorsValue::None,
            max_age: None,
        })
    }

    pub fn allow_any_origin(mut self) -> Self {
        self.0.allowed_origins = CorsOrigin::Any;
        self
    }

    pub fn allow_origins<I, T>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let origins = origins.into_iter().map(|s| s.into()).collect::<_>();
        self.0.allowed_origins = CorsOrigin::List(origins);
        self
    }

    pub fn with_origin<F>(mut self, f: F) -> Self
    where
        F: Fn(&Request<Body>) -> bool + Send + Sync + 'static,
    {
        self.0.allowed_origins = CorsOrigin::Dynamic(Box::new(f));
        self
    }

    pub fn allow_any_method(mut self) -> Self {
        self.0.allowed_methods = CorsValue::Any;
        self
    }

    pub fn allow_method(mut self, method: Method) -> Self {
        match &mut self.0.allowed_methods {
            CorsValue::List(list) => {
                list.insert(method);
            }
            _ => {
                let values = HashSet::from_iter([method]);
                let _ = std::mem::replace(&mut self.0.allowed_methods, CorsValue::List(values));
            }
        }

        self
    }

    pub fn allow_methods<I>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = Method>,
    {
        match &mut self.0.allowed_methods {
            CorsValue::List(list) => {
                list.extend(methods);
            }
            _ => {
                let values = HashSet::from_iter(methods);
                let _ = std::mem::replace(&mut self.0.allowed_methods, CorsValue::List(values));
            }
        }

        self
    }

    pub fn allow_any_header(mut self) -> Self {
        self.0.allowed_headers = CorsValue::Any;
        self
    }

    pub fn allow_header(mut self, header_name: HeaderName) -> Self {
        match &mut self.0.allowed_headers {
            CorsValue::List(list) => {
                list.insert(header_name);
            }
            _ => {
                let values = HashSet::from_iter([header_name]);
                let _ = std::mem::replace(&mut self.0.allowed_headers, CorsValue::List(values));
            }
        }

        self
    }

    pub fn allow_any_expose_headers(mut self) -> Self {
        self.0.expose_headers = CorsValue::Any;
        self
    }

    pub fn allow_expose_header(mut self, header_name: HeaderName) -> Self {
        match &mut self.0.expose_headers {
            CorsValue::List(list) => {
                list.insert(header_name);
            }
            _ => {
                let values = HashSet::from_iter([header_name]);
                let _ = std::mem::replace(&mut self.0.expose_headers, CorsValue::List(values));
            }
        }

        self
    }

    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.0.allow_credentials = Some(allow_credentials);
        self
    }

    pub fn max_age(mut self, max_age: u64) -> Self {
        self.0.max_age = Some(max_age);
        self
    }

    pub fn build(self) -> Cors {
        self.0
    }
}

impl Default for CorsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Cors {
    fn on_request(
        &self,
        req: Request<Body>,
        next: &crate::handler::BoxedHandler,
    ) -> http1::response::Response<Body> {
        let cors_headers = get_cors_headers(self, &req);
        let mut outgoing_response = if req.method() == Method::OPTIONS {
            Response::new(StatusCode::NO_CONTENT, Body::empty())
        } else {
            next.call(req)
        };

        outgoing_response.headers_mut().extend(cors_headers);
        outgoing_response
    }
}

fn get_comma_separated_list<I: IntoIterator<Item = T>, T: Display>(iter: I) -> String {
    iter.into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn get_cors_headers(cors: &Cors, req: &Request<Body>) -> Headers {
    let mut headers = Headers::new();

    match &cors.allowed_origins {
        CorsOrigin::Any => {
            headers.insert(
                headers::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
        }
        CorsOrigin::List(list) => {
            if let Some(origin) = req.headers().get(headers::ORIGIN) {
                if list.contains(origin.as_str()) {
                    headers.insert(
                        headers::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_string(origin.to_string()),
                    );
                }
            }
        }
        CorsOrigin::Dynamic(f) => {
            if let Some(origin) = req.headers().get(headers::ORIGIN) {
                let is_allowed = f(req);

                if is_allowed {
                    headers.insert(
                        headers::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_string(origin.to_string()),
                    );
                }
            }
        }
    }

    match &cors.allowed_methods {
        CorsValue::None => {}
        CorsValue::Any => {
            headers.insert(
                headers::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("*"),
            );
        }
        CorsValue::List(list) => {
            let methods = get_comma_separated_list(list);
            headers.insert(
                headers::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_string(methods),
            );
        }
    }

    match &cors.allowed_headers {
        CorsValue::None => {}
        CorsValue::Any => {
            headers.insert(
                headers::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("*"),
            );
        }
        CorsValue::List(list) => {
            let headers_str = get_comma_separated_list(list);
            headers.insert(
                headers::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_string(headers_str),
            );
        }
    }

    match &cors.expose_headers {
        CorsValue::None => {}
        CorsValue::Any => {
            headers.insert(
                headers::ACCESS_CONTROL_EXPOSE_HEADERS,
                HeaderValue::from_static("*"),
            );
        }
        CorsValue::List(list) => {
            let headers_str = get_comma_separated_list(list);
            headers.insert(
                headers::ACCESS_CONTROL_EXPOSE_HEADERS,
                HeaderValue::from_string(headers_str),
            );
        }
    }

    if let Some(allow_credentials) = cors.allow_credentials {
        headers.insert(
            headers::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderValue::from_static(if allow_credentials { "true" } else { "false" }),
        );
    }

    if let Some(max_age) = cors.max_age {
        headers.insert(headers::ACCESS_CONTROL_MAX_AGE, max_age);
    }

    headers
}
