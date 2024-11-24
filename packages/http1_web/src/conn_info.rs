use crate::{from_request::FromRequest, ErrorStatusCode, IntoResponse};
use http1::{body::Body, protocol::connection::Connected, response::Response};
use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

/// Extracts connection information from the server request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionInfo<T>(pub T);

impl<T> ConnectionInfo<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ConnectionInfo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ConnectionInfo<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: FromConnectionInfo> FromRequest for ConnectionInfo<T> {
    type Rejection = Response<Body>;

    fn from_request(
        req: &http1::request::Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        match req.extensions().get::<Connected>() {
            Some(conn) => {
                let value = match T::from_connection_info(conn) {
                    Ok(x) => x,
                    Err(err) => return Err(err.into_response()),
                };

                Ok(ConnectionInfo(value))
            }
            None => {
                log::warn!("Configure the server to expose the connection information with Server.include_conn_info(true)");
                Err(ErrorStatusCode::InternalServerError.into_response())
            }
        }
    }
}

/// Extract incoming request connection information.
#[doc(hidden)]
pub trait FromConnectionInfo: Sized {
    type Rejection: IntoResponse;

    fn from_connection_info(conn: &Connected) -> Result<Self, Self::Rejection>;
}

impl FromConnectionInfo for SocketAddr {
    type Rejection = ErrorStatusCode;

    fn from_connection_info(conn: &Connected) -> Result<Self, Self::Rejection> {
        match conn.peer_addr() {
            Some(addr) => Ok(addr),
            None => {
                log::error!("Failed to resolve ip address");
                Err(ErrorStatusCode::InternalServerError)
            }
        }
    }
}
