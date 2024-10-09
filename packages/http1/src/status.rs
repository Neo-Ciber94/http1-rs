use std::fmt::Display;

// Represents an status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Constructs an status code.
    pub const fn new(status: u16) -> Self {
        // https://www.rfc-editor.org/rfc/rfc9110.html#name-status-codes
        if status < 100 || status > 599 {
            panic!("Invalid status code, expected value between 100 and 599")
        }

        StatusCode(status)
    }

    /// Returns the status as `u16`.
    pub const fn as_u16(&self) -> u16 {
        self.0
    }

    /// Whether if this status is a redirections status code.
    pub fn is_redirection(&self) -> bool {
        self.0 >= 300 && self.0 < 400
    }

    /// Whether if this status is a client error status code.
    pub fn is_client_error(&self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    /// Whether if this status is a server error status code.
    pub fn is_server_error(&self) -> bool {
        self.0 >= 500
    }
}

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode::OK
    }
}

macro_rules! status_codes {
    ($($status_code:expr, $reason_phrase:expr, $kconst:ident),*) => {
        impl StatusCode {
            $(
                pub const $kconst: StatusCode = StatusCode($status_code);
            )*

            /// Returns the reason for this status code.
            pub fn reason_phrase(&self) -> Option<&str> {
                match self.0 {
                    $(
                        $status_code => Some($reason_phrase),
                    )*
                    _ => None
                }
            }
        }
    };
}

status_codes! {
    // Successful 2xx
    200, "OK", OK,
    201, "Created", CREATED,
    202, "Accepted", ACCEPTED,
    203, "Non-Authoritative Information", NON_AUTHORITATIVE_INFORMATION,
    204, "No Content", NO_CONTENT,
    205, "Reset Content", RESET_CONTENT,
    206, "Partial Content", PARTIAL_CONTENT,

    // Redirection 3xx
    300, "Multiple Choices", MULTIPLE_CHOICES,
    301, "Moved Permanently", MOVED_PERMANENTLY,
    302, "Found", FOUND,
    303, "See Other", SEE_OTHER,
    304, "Not Modified", NOT_MODIFIED,
    305, "Use Proxy", USE_PROXY,
    306, "(Unused)", UNUSED,
    307, "Temporary Redirect", TEMPORARY_REDIRECT,
    308, "Permanent Redirect", PERMANENT_REDIRECT,

    // Client Error 4xx
    400, "Bad Request", BAD_REQUEST,
    401, "Unauthorized", UNAUTHORIZED,
    402, "Payment Required", PAYMENT_REQUIRED,
    403, "Forbidden", FORBIDDEN,
    404, "Not Found", NOT_FOUND,
    405, "Method Not Allowed", METHOD_NOT_ALLOWED,
    406, "Not Acceptable", NOT_ACCEPTABLE,
    407, "Proxy Authentication Required", PROXY_AUTHENTICATION_REQUIRED,
    408, "Request Timeout", REQUEST_TIMEOUT,
    409, "Conflict", CONFLICT,
    410, "Gone", GONE,
    411, "Length Required", LENGTH_REQUIRED,
    412, "Precondition Failed", PRECONDITION_FAILED,
    413, "Payload Too Large", PAYLOAD_TOO_LARGE,
    414, "URI Too Long", URI_TOO_LONG,
    415, "Unsupported Media Type", UNSUPPORTED_MEDIA_TYPE,
    416, "Range Not Satisfiable", RANGE_NOT_SATISFIABLE,
    417, "Expectation Failed", EXPECTATION_FAILED,
    418, "I'm a Teapot", IM_A_TEAPOT,
    421, "Misdirected Request", MISDIRECTED_REQUEST,
    422, "Unprocessable Content", UNPROCESSABLE_CONTENT,
    423, "Locked", LOCKED,
    424, "Failed Dependency", FAILED_DEPENDENCY,
    425, "Too Early", TOO_EARLY,
    426, "Upgrade Required", UPGRADE_REQUIRED,
    428, "Precondition Required", PRECONDITION_REQUIRED,
    429, "Too Many Requests", TOO_MANY_REQUESTS,
    431, "Request Header Fields Too Large", REQUEST_HEADER_FIELDS_TOO_LARGE,
    451, "Unavailable For Legal Reasons", UNAVAILABLE_FOR_LEGAL_REASONS,

    // Server Error 5xx
    500, "Internal Server Error", INTERNAL_SERVER_ERROR,
    501, "Not Implemented", NOT_IMPLEMENTED,
    502, "Bad Gateway", BAD_GATEWAY,
    503, "Service Unavailable", SERVICE_UNAVAILABLE,
    504, "Gateway Timeout", GATEWAY_TIMEOUT,
    505, "HTTP Version Not Supported", HTTP_VERSION_NOT_SUPPORTED,
    506, "Variant Also Negotiates", VARIANT_ALSO_NEGOTIATES,
    507, "Insufficient Storage", INSUFFICIENT_STORAGE,
    508, "Loop Detected", LOOP_DETECTED,
    510, "Not Extended", NOT_EXTENDED,
    511, "Network Authentication Required", NETWORK_AUTHENTICATION_REQUIRED
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
