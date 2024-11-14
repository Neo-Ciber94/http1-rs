use std::fmt::Display;

use http1::{
    body::Body, common::base64::encode, error::BoxError, headers, method::Method,
    protocol::upgrade::Upgrade, request::Request, response::Response, status::StatusCode,
};

use crate::{from_request::FromRequestRef, IntoResponse};

use super::WebSocket;

const WEB_SOCKET_VERSION: &str = "13";
const WEB_SOCKET_UUID_STR: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
pub struct WebSocketUpgrade {
    key: String,
}

impl WebSocketUpgrade {
    pub fn upgrade(self, mut req: Request<Body>) -> Result<(WebSocket, Response<Body>), BoxError> {
        let WebSocketUpgrade { key, .. } = self;
        let hash_bytes = http1::common::sha1::hash(format!("{key}{WEB_SOCKET_UUID_STR}"));
        let hash_str = String::from_utf8(hash_bytes)?;

        let upgrade = req
            .extensions_mut()
            .remove::<Upgrade>()
            .ok_or_else(|| format!("Failed to get connection upgrade stream"))?;

        let web_socket = WebSocket::new(upgrade);
        let accept_key = encode(hash_str);

        let response = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .insert_header(headers::UPGRADE, "websocket")
            .insert_header(headers::CONNECTION, "upgrade")
            .insert_header(headers::SEC_WEBSOCKET_ACCEPT, accept_key)
            .body(Body::empty());

        Ok((web_socket, response))
    }
}

#[derive(Debug)]
pub enum WebSocketUpgradeError {
    InvalidMethod(Method),
    MissingHost,
    MissingUpgrade,
    MissingConnectionUpgrade,
    MissingKey,
    MissingVersion,
    NoProtocolsSupported(String),
    NoExtensionsSupported(String),
    InvalidVersion(String),
    InvalidKey(String),
    Other(BoxError),
}

impl std::error::Error for WebSocketUpgradeError {}

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
            WebSocketUpgradeError::InvalidVersion(version) => write!(
                f,
                "Invalid version expected `{WEB_SOCKET_VERSION}` but was `{version}`"
            ),
            WebSocketUpgradeError::InvalidKey(key) => write!(
                f,
                "Invalid key: `{key}`"
            ),
            WebSocketUpgradeError::Other(error) => write!(f, "{error}"),
            WebSocketUpgradeError::NoProtocolsSupported(protocols) => write!(
                f,
                "`Sec-WebSocket-Protocol` was found but not protocol are supported: {protocols}"
            ),
            WebSocketUpgradeError::NoExtensionsSupported(extensions) => write!(
                f,
                "`Sec-WebSocket-Extensions` was found but not extensions are supported: {extensions}"
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

        if let Some(upgrade) = headers.get(headers::UPGRADE) {
            if upgrade.as_str().eq_ignore_ascii_case("websocket") {
                return Err(WebSocketUpgradeError::MissingUpgrade);
            }
        }

        if let Some(conn) = headers.get(headers::CONNECTION) {
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

        if web_socket_version != WEB_SOCKET_VERSION {
            return Err(WebSocketUpgradeError::InvalidVersion(
                web_socket_version.as_str().to_string(),
            ));
        }

        let key = http1::common::base64::decode(web_socket_key).map_err(|err| {
            WebSocketUpgradeError::Other(format!("Failed to parse websocket key: {err}").into())
        })?;

        if key.as_bytes().len() != 13 {
            return Err(WebSocketUpgradeError::InvalidKey(key));
        }

        if let Some(protocols) = headers.get(headers::SEC_WEBSOCKET_PROTOCOL) {
            return Err(WebSocketUpgradeError::NoProtocolsSupported(
                protocols.to_string(),
            ));
        }

        if let Some(exts) = headers.get(headers::SEC_WEBSOCKET_EXTENSIONS) {
            return Err(WebSocketUpgradeError::NoExtensionsSupported(
                exts.to_string(),
            ));
        }

        Ok(WebSocketUpgrade { key })
    }
}
