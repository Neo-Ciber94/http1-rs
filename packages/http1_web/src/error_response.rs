use http1::{body::Body, response::Response, status::StatusCode};

use crate::into_response::IntoResponse;

/// Represents a client or server error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStatusCode {
    // 400 Series (Client Errors)
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    ImATeapot,
    MisdirectedRequest,
    UnprocessableContent,
    Locked,
    FailedDependency,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge,
    UnavailableForLegalReasons,

    // 500 Series (Server Errors)
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    NotExtended,
    NetworkAuthenticationRequired,
}

impl ErrorStatusCode {
    pub fn as_status_code(&self) -> StatusCode {
        match self {
            // 400 Series (Client Errors)
            ErrorStatusCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorStatusCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorStatusCode::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
            ErrorStatusCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorStatusCode::NotFound => StatusCode::NOT_FOUND,
            ErrorStatusCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ErrorStatusCode::NotAcceptable => StatusCode::NOT_ACCEPTABLE,
            ErrorStatusCode::ProxyAuthenticationRequired => {
                StatusCode::PROXY_AUTHENTICATION_REQUIRED
            }
            ErrorStatusCode::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            ErrorStatusCode::Conflict => StatusCode::CONFLICT,
            ErrorStatusCode::Gone => StatusCode::GONE,
            ErrorStatusCode::LengthRequired => StatusCode::LENGTH_REQUIRED,
            ErrorStatusCode::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            ErrorStatusCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorStatusCode::URITooLong => StatusCode::URI_TOO_LONG,
            ErrorStatusCode::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ErrorStatusCode::RangeNotSatisfiable => StatusCode::RANGE_NOT_SATISFIABLE,
            ErrorStatusCode::ExpectationFailed => StatusCode::EXPECTATION_FAILED,
            ErrorStatusCode::ImATeapot => StatusCode::IM_A_TEAPOT,
            ErrorStatusCode::MisdirectedRequest => StatusCode::MISDIRECTED_REQUEST,
            ErrorStatusCode::UnprocessableContent => StatusCode::UNPROCESSABLE_CONTENT,
            ErrorStatusCode::Locked => StatusCode::LOCKED,
            ErrorStatusCode::FailedDependency => StatusCode::FAILED_DEPENDENCY,
            ErrorStatusCode::UpgradeRequired => StatusCode::UPGRADE_REQUIRED,
            ErrorStatusCode::PreconditionRequired => StatusCode::PRECONDITION_REQUIRED,
            ErrorStatusCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorStatusCode::RequestHeaderFieldsTooLarge => {
                StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE
            }
            ErrorStatusCode::UnavailableForLegalReasons => {
                StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS
            }

            // 500 Series (Server Errors)
            ErrorStatusCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorStatusCode::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ErrorStatusCode::BadGateway => StatusCode::BAD_GATEWAY,
            ErrorStatusCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ErrorStatusCode::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            ErrorStatusCode::HTTPVersionNotSupported => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            ErrorStatusCode::VariantAlsoNegotiates => StatusCode::VARIANT_ALSO_NEGOTIATES,
            ErrorStatusCode::InsufficientStorage => StatusCode::INSUFFICIENT_STORAGE,
            ErrorStatusCode::LoopDetected => StatusCode::LOOP_DETECTED,
            ErrorStatusCode::NotExtended => StatusCode::NOT_EXTENDED,
            ErrorStatusCode::NetworkAuthenticationRequired => {
                StatusCode::NETWORK_AUTHENTICATION_REQUIRED
            }
        }
    }
}

impl IntoResponse for ErrorStatusCode {
    fn into_response(self) -> Response<Body> {
        self.as_status_code().into_response()
    }
}

enum Inner {
    Response(Box<dyn FnOnce() -> Response<Body>>),
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Represents an error response.
pub struct ErrorResponse {
    status: ErrorStatusCode,
    inner: Inner,
}

impl ErrorResponse {
    /// Constructs an `ErrorResponse` with a type that can be converted to a `IntoResponse`.
    pub fn new<T>(status: ErrorStatusCode, response: T) -> Self
    where
        T: IntoResponse + 'static,
    {
        let response = Box::new(|| response.into_response());
        ErrorResponse {
            status,
            inner: Inner::Response(response),
        }
    }

    /// Constructs an `ErrorResponse` from a status an an `std::error::Error`.
    pub fn from_error<E>(status: ErrorStatusCode, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        ErrorResponse {
            status,
            inner: Inner::Error(error.into()),
        }
    }

    /// Returns the status code for this error.
    pub fn status_code(&self) -> StatusCode {
        self.status.as_status_code()
    }

    /// Returns the error status code.
    pub fn error_status(&self) -> ErrorStatusCode {
        self.status
    }

    /// Returns the error used for this response if any.
    pub fn error(&self) -> Option<&Box<dyn std::error::Error + Send + Sync + 'static>> {
        match &self.inner {
            Inner::Response(_) => None,
            Inner::Error(error) => Some(error),
        }
    }

    /// Returns whether if this is a client error (400..499).
    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }

    /// Returns whether if this is a server error (500..599).
    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let status_code = self.status_code().clone();

        match self.inner {
            Inner::Response(f) => {
                let mut response = f();
                *response.status_mut() = status_code;
                response
            }
            Inner::Error(error) => {
                let msg = error.to_string();
                Response::new(status_code, msg.into())
            }
        }
    }
}

impl From<ErrorStatusCode> for ErrorResponse {
    fn from(value: ErrorStatusCode) -> Self {
        ErrorResponse::new(value, ())
    }
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>> From<E> for ErrorResponse {
    fn from(value: E) -> Self {
        fn error_status(
            err: &Box<dyn std::error::Error + Send + Sync + 'static>,
        ) -> ErrorStatusCode {
            if let Some(err) = err.downcast_ref::<std::io::Error>() {
                match err.kind() {
                    std::io::ErrorKind::NotFound => ErrorStatusCode::NotFound, // 404
                    std::io::ErrorKind::PermissionDenied => ErrorStatusCode::Forbidden, // 403
                    std::io::ErrorKind::AddrInUse => ErrorStatusCode::Conflict, // 409
                    std::io::ErrorKind::AlreadyExists => ErrorStatusCode::Conflict, // 409
                    std::io::ErrorKind::InvalidInput => ErrorStatusCode::BadRequest, // 400
                    std::io::ErrorKind::InvalidData => ErrorStatusCode::BadRequest, // 400
                    std::io::ErrorKind::WouldBlock => ErrorStatusCode::RequestTimeout, // 408
                    _ => ErrorStatusCode::InternalServerError, // Default to 500 for any other
                }
            } else {
                ErrorStatusCode::InternalServerError
            }
        }

        let error = value.into();
        let status = error_status(&error);
        ErrorResponse::from_error(status, error)
    }
}
