use std::fmt::Display;

use http1::{headers, method::Method, status::StatusCode};

use crate::{from_request::FromRequestRef, IntoResponse};

const WEB_SOCKET_VERSION: u8 = 13;

pub struct WebSocketUpgrade;

#[derive(Debug)]
pub enum WebSocketUpgradeError {
    InvalidMethod(Method),
    MissingHost,
    MissingUpgrade,
    MissingConnectionUpgrade,
    MissingKey,
    MissingVersion,
    MissingProtocol,
    MissingExtensions,
    InvalidVersion,
}

impl Display for WebSocketUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketUpgradeError::InvalidMethod(method) => {
                write!(f, "invalid http method: `{method}` expected GET")
            }
            WebSocketUpgradeError::MissingHost => write!(f, "Missing `Host` header"),
            WebSocketUpgradeError::MissingUpgrade => {
                write!(f, "Missing `Upgrade: websocket` header")
            }
            WebSocketUpgradeError::MissingConnectionUpgrade => {
                write!(f, "Missing `Connection: websocket` header")
            }
            WebSocketUpgradeError::MissingKey => {
                write!(f, "Missing websocket `Sec-WebSocket-Key` header")
            }
            WebSocketUpgradeError::MissingVersion => {
                write!(f, "Missing `Sec-WebSocket-Version` header")
            }
            WebSocketUpgradeError::MissingProtocol => {
                write!(f, "Missing `Sec-WebSocket-Protocol` header")
            }
            WebSocketUpgradeError::MissingExtensions => {
                write!(f, "Missing `Sec-WebSocket-Extensions` header")
            }
            WebSocketUpgradeError::InvalidVersion => write!(
                f,
                "Invalid websocket version, expected: {WEB_SOCKET_VERSION}"
            ),
        }
    }
}

impl IntoResponse for WebSocketUpgradeError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("Failed to upgrade websocket connection: {self}");
        StatusCode::BAD_REQUEST.into_response()
    }
}

impl FromRequestRef for WebSocketUpgrade {
    type Rejection = WebSocketUpgradeError;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        // Parsing the websocket according to:
        // https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.1
        if req.method() != Method::GET {
            return Err(WebSocketUpgradeError::InvalidMethod(req.method().clone()));
        }

        let headers = req.headers();
        let upgrade_header = headers.get(headers::UPGRADE);

        if let Some(upgrade) = upgrade_header {
            if upgrade.as_str().eq_ignore_ascii_case("websocket") {
                return Err(WebSocketUpgradeError::MissingUpgrade);
            }
        }

        let connection_header = headers.get(headers::CONNECTION);

        if let Some(conn) = connection_header {
            if conn.as_str().eq_ignore_ascii_case("Upgrade") {
                return Err(WebSocketUpgradeError::MissingConnectionUpgrade);
            }
        }

        let web_socket_key = headers
            .get(headers::SEC_WEBSOCKET_KEY)
            .ok_or(WebSocketUpgradeError::MissingKey)?;
        let web_socket_version = headers
            .get(headers::SEC_WEBSOCKET_VERSION)
            .ok_or(WebSocketUpgradeError::MissingVersion)?;
        let web_socket_protocol = headers
            .get(headers::SEC_WEBSOCKET_PROTOCOL)
            .ok_or(WebSocketUpgradeError::MissingVersion)?;
        let web_socket_exts = headers
            .get(headers::SEC_WEBSOCKET_EXTENSIONS)
            .ok_or(WebSocketUpgradeError::MissingExtensions)?;

        todo!()
    }
}
