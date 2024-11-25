use http1::{body::Body, response::Response};
use http1_web::{
    cookies::Cookies, from_request::FromRequest, redirect::Redirect, state::State, ErrorStatusCode,
    IntoResponse,
};

use crate::{consts::COOKIE_SESSION_NAME, models::User};
use app::db::KeyValueDatabase;

#[derive(Debug)]
pub struct AuthenticatedUser(pub User);

pub enum AuthenticatedUserRejection {
    StateError,
    RedirectToLogin,
}

impl IntoResponse for AuthenticatedUserRejection {
    fn into_response(self) -> Response<Body> {
        match self {
            AuthenticatedUserRejection::StateError => {
                ErrorStatusCode::InternalServerError.into_response()
            }
            AuthenticatedUserRejection::RedirectToLogin => {
                Redirect::see_other("/login").into_response()
            }
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Rejection = AuthenticatedUserRejection;

    fn from_request(
        req: &http1::request::Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        let State(db) = State::<KeyValueDatabase>::from_request(req, extensions, payload)
            .inspect_err(|err| {
                log::error!("Failed to get database: {err}");
            })
            .map_err(|_| AuthenticatedUserRejection::StateError)?;

        let cookies = Cookies::from_headers(req.headers()).unwrap_or_default();

        let session_cookie = cookies
            .get(COOKIE_SESSION_NAME)
            .ok_or(AuthenticatedUserRejection::RedirectToLogin)?;

        let session_id = session_cookie.value();

        match crate::models::get_session_user(&db, session_id.to_owned()) {
            Ok(Some(user)) => Ok(AuthenticatedUser(user)),
            x => {
                log::error!("user session `{session_id}` not found: {x:?}");
                Err(AuthenticatedUserRejection::RedirectToLogin)
            }
        }
    }
}
