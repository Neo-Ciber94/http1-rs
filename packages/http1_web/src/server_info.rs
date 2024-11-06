use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use http1::{body::Body, response::Response, server::Config};

use crate::{from_request::FromRequest, ErrorStatusCode, IntoResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInfo<T>(pub T);

impl<T> ServerInfo<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ServerInfo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ServerInfo<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: FromServerConfig> FromRequest for ServerInfo<T> {
    type Rejection = Response<Body>;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        match req.extensions().get::<http1::server::Config>() {
            Some(config) => {
                let value = T::from_server_config(config).map_err(|e| e.into_response())?;
                Ok(ServerInfo(value))
            }
            None => {
                log::warn!("Configure the server to expose the connection information with Server.include_server_info(true)");
                Err(ErrorStatusCode::InternalServerError.into_response())
            }
        }
    }
}

/// Extract information from the server config.
pub trait FromServerConfig: Sized {
    type Rejection: IntoResponse;
    fn from_server_config(config: &Config) -> Result<Self, Self::Rejection>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BodySizeLimit(Option<usize>);

impl BodySizeLimit {
    pub fn has_limit(&self) -> bool {
        self.0.is_some()
    }

    pub fn bytes_limit(&self) -> Option<usize> {
        self.0
    }
}

impl FromServerConfig for BodySizeLimit {
    type Rejection = Infallible;

    fn from_server_config(config: &Config) -> Result<Self, Self::Rejection> {
        Ok(BodySizeLimit(config.max_body_size))
    }
}
