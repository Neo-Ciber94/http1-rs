use std::time::Duration;

use datetime::DateTime;
use http1::{body::Body, error::BoxError, headers, response::Response};
use http1_web::{
    app::Scope,
    cookies::{Cookie, Cookies},
    forms::form::Form,
    path::Path,
    redirect::Redirect,
    state::State,
    ErrorResponse, ErrorStatusCode, HttpResponse, IntoResponse,
};

use super::models::AuthenticatedUser;
use crate::{
    components::{AlertKind, AlertProps},
    consts::{COOKIE_FLASH_MESSAGE, COOKIE_SESSION_NAME},
    models::ValidationError,
};
use app::db::KeyValueDatabase;

macro_rules! tri {
    ($location:expr, $body:expr) => {{
        let mut res = HttpResponse::see_other("/todos");

        match $body {
            Err(err) if err.is::<ValidationError>() => {
                let err = err.downcast::<ValidationError>().unwrap();
                res.headers_mut().insert(
                    headers::LOCATION,
                    headers::HeaderValue::try_from($location).unwrap(),
                );

                if let Err(err) = set_flash_message(&mut res, AlertKind::Error, err.to_string()) {
                    return Err(err.into());
                }

                res.finish()
            }
            Err(err) => return Err(err.into()),
            _ => res.finish(),
        }
    }};
}

#[derive(Debug)]
struct LoginUser {
    pub username: String,
}

serde::impl_serde_struct!(LoginUser => {
     username: String
});

struct CreateTodo {
    pub title: String,
    pub description: Option<String>,
}

serde::impl_serde_struct!(CreateTodo => {
     title: String,
     description: Option<String>,
});

struct UpdateTodo {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
}

serde::impl_serde_struct!(UpdateTodo => {
     id: u64,
     title: String,
     description: Option<String>,
});

pub fn api_routes() -> Scope {
    Scope::new()
        .post(
            "/login",
            |State(db): State<KeyValueDatabase>,
             Form(input): Form<LoginUser>|
             -> Result<HttpResponse, ErrorResponse> {
                let user = match crate::models::get_user_by_username(&db, &input.username)? {
                    Some(x) => x,
                    None => {
                        let new_user = crate::models::insert_user(&db, input.username)?;
                        log::info!("User created: {new_user:?}");
                        new_user
                    }
                };

                let session = crate::models::create_session(&db, user.id)?;
                let session_cookie = Cookie::new(COOKIE_SESSION_NAME, session.id.clone())
                    .path("/")
                    .http_only(true)
                    .expires(DateTime::now_utc() + Duration::from_secs(60 * 60))
                    .build();

                Ok(HttpResponse::see_other("/todos")
                    .set_cookie(session_cookie)
                    .finish())
            },
        )
        .get(
            "/logout",
            |State(db): State<KeyValueDatabase>,
             mut cookies: Cookies,
             auth: Option<AuthenticatedUser>|
             -> Result<Response<Body>, ErrorResponse> {
                if auth.is_none() {
                    return Ok(Redirect::see_other("/todos").into_response());
                }

                let session_cookie = match cookies.get(COOKIE_SESSION_NAME) {
                    Some(c) => c,
                    None => {
                        return Ok(Redirect::see_other("/todos").into_response());
                    }
                };

                crate::models::remove_session(&db, session_cookie.value().to_owned())?;
                cookies.del(crate::consts::COOKIE_SESSION_NAME);

                Ok((Redirect::see_other("/"), cookies).into_response())
            },
        )
        .post(
            "/todos/create",
            |State(db): State<KeyValueDatabase>,
             AuthenticatedUser(user): AuthenticatedUser,
             Form(todo): Form<CreateTodo>|
             -> Result<HttpResponse, ErrorResponse> {
                Ok(tri!(
                    "/todos/create",
                    crate::models::insert_todo(
                        &db,
                        todo.title,
                        todo.description.filter(|x| !x.is_empty()),
                        user.id,
                    )
                ))
            },
        )
        .post(
            "/todos/update",
            |State(db): State<KeyValueDatabase>,
             Form(form): Form<UpdateTodo>|
             -> Result<HttpResponse, ErrorResponse> {
                let mut todo = match crate::models::get_todo(&db, form.id)? {
                    Some(x) => x,
                    None => return Err(ErrorStatusCode::NotFound.into()),
                };

                // Update fields
                todo.title = form.title;
                todo.description = form.description;

                // Save
                let todo_id = todo.id;
                let mut res = tri!(
                    format!("/todos/edit/{todo_id}"),
                    crate::models::update_todo(&db, todo)
                );
                set_flash_message(&mut res, AlertKind::Success, "Successfully updated")?;
                Ok(res)
            },
        )
        .post(
            "/todos/delete/:todo_id",
            |State(db): State<KeyValueDatabase>,
             Path(todo_id): Path<u64>|
             -> Result<Redirect, ErrorResponse> {
                crate::models::delete_todo(&db, todo_id)?;

                Ok(Redirect::see_other("/todos"))
            },
        )
        .post(
            "/todos/toggle/:todo_id",
            |State(db): State<KeyValueDatabase>,
             Path(todo_id): Path<u64>|
             -> Result<Redirect, ErrorResponse> {
                let mut todo = match crate::models::get_todo(&db, todo_id)? {
                    Some(x) => x,
                    None => return Err(ErrorStatusCode::NotFound.into()),
                };

                todo.is_done = !todo.is_done;
                crate::models::update_todo(&db, todo)?;

                Ok(Redirect::see_other("/todos"))
            },
        )
}

fn set_flash_message<B>(
    res: &mut HttpResponse<B>,
    kind: AlertKind,
    message: impl Into<String>,
) -> Result<(), BoxError> {
    use http1::headers::HeaderValue;

    let json = serde::json::to_string(&AlertProps::new(message, kind))?;
    let cookie = Cookie::new(COOKIE_FLASH_MESSAGE, json)
        .max_age(1)
        .build()
        .to_string();

    res.headers_mut()
        .insert(headers::SET_COOKIE, HeaderValue::from_string(cookie));
    Ok(())
}
