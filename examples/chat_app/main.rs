use std::sync::{Arc, Mutex, RwLock};

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
    let chat_room = ChatRoom(Arc::new(RwLock::new(Default::default())));

    server
        .on_ready(|addr| log::debug!("Listening on http://{addr}"))
        .start(
            App::new()
                .middleware(Logging)
                .middleware(auth_middleware)
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
    let path = req.uri().path_and_query().path();

    if path.starts_with("/api") {
        return next.call(req);
    }

    let is_authenticated = http1_web::guard!(req.extract::<Option<ChatUser>>()).is_some();
    let to_login = path == "/login";
    log::debug!("is_authenticated? {is_authenticated}, path: {path}");

    if is_authenticated && to_login {
        Redirect::see_other("/").into_response()
    } else if !is_authenticated && !to_login {
        Redirect::see_other("/login").into_response()
    } else {
        next.call(req)
    }
}
