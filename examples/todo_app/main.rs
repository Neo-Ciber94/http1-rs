mod components;
mod consts;
mod models;
mod routes;

use app::db::KeyValueDatabase;
use http1::server::Server;
use http1_web::{
    app::App,
    fs::ServeDir,
    middleware::{logging::Logging, redirection::Redirection},
};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "localhost:5000";

    let app = App::new()
        .state(KeyValueDatabase::new("examples/todo_app/db.json").unwrap())
        .middleware(Logging)
        .middleware(Redirection::new("/", "/login"))
        .get("/*", ServeDir::new("examples/todo_app/public"))
        .scope("/api", crate::routes::api::api_routes())
        .scope("/", crate::routes::pages::page_routes());

    Server::new(addr)
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .start(app)
}
