enum CorsOrigin {
    Any,
    Static(Vec<String>),
    Dynamic(Box<dyn Fn() -> String>),
}

#[derive(Debug)]
enum CorsValue<T> {
    None,
    Any,
    List(HashSet<T>),
}

impl Debug for CorsOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorsOrigin::Any => f.debug_tuple("Any").finish(),
            CorsOrigin::Static(x) => f.debug_tuple("Static").field(x).finish(),
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

    pub fn any_origin(mut self) -> Self {
        self.0.allowed_origins = CorsOrigin::Any;
        self
    }

    pub fn origin(mut self) -> Self {
        self.0.allowed_origins = self
    }
}
