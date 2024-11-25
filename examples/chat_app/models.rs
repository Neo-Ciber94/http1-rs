use http1::common::uuid::Uuid;
use http1_web::{cookies::Cookies, from_request::FromRequest, ErrorStatusCode};
use serde::impl_serde_struct;

use crate::constants;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Uuid,
    pub username: String,
    pub content: String,
}

impl_serde_struct!(ChatMessage => {
    id: Uuid,
    username: String,
    content: String,
});

#[derive(Debug, Clone)]
pub struct ChatUser {
    pub username: String,
}

impl_serde_struct!(ChatUser => {
    username: String
});

impl FromRequest for ChatUser {
    type Rejection = ErrorStatusCode;

    fn from_request(
        req: &http1::request::Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request(req, extensions, payload).unwrap_or_default();

        let Some(cookie) = cookies.get(constants::COOKIE_AUTH_SESSION) else {
            return Err(ErrorStatusCode::Unauthorized);
        };

        let username = http1::common::base64::decode(cookie.value())
            .map_err(|_| ErrorStatusCode::Unauthorized)?;

        Ok(ChatUser { username })
    }
}
