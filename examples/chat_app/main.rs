use std::sync::{Arc, Mutex};

use app::db::KeyValueDatabase;
use http1::{body::Body, request::Request, response::Response, server::Server};
use http1_web::{
    app::App, fs::ServeDir, handler::BoxedHandler, middleware::logging::Logging,
    redirect::Redirect, IntoResponse, RequestExt,
};
use models::{ChatRoom, ChatUser};

mod constants;
mod models;
mod routes;

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let server = Server::new("localhost:5678");
    let chat_room = ChatRoom(Arc::new(Mutex::new(vec![])));

    server
        .on_ready(|addr| log::debug!("Listening on http://{addr}"))
        .start(
            App::new()
                .middleware(Logging)
                //.middleware(auth_middleware)
                .state(KeyValueDatabase::new("examples/chat_app/db.json").unwrap())
                .state(chat_room)
                .scope("/api", routes::api::routes())
                .get(
                    "/*",
                    ServeDir::new("examples/chat_app/static").append_html_index(true),
                ),
        )
}

fn auth_middleware(req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
    let is_authenticated = http1_web::guard!(req.extract::<Option<ChatUser>>()).is_some();
    let path = req.uri().path_and_query().path();
    let is_login = path == "/login";

    if is_authenticated && is_login {
        Redirect::see_other("/").into_response()
    } else if !is_authenticated && !is_login {
        Redirect::see_other("/login").into_response()
    } else {
        next.call(req)
    }
}
