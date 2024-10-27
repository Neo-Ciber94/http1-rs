mod components;
mod consts;
mod db;
mod models;
mod routes;

use db::KeyValueDatabase;
use http1::server::Server;
use http1_web::{
    app::App,
    fs::ServeDir,
    middleware::{logging::Logging, redirection::Redirection},
};
use routes::{api_routes, page_routes};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "localhost:5000";

    let app = App::new()
        .state(KeyValueDatabase::new("examples/todo_app/db.json").unwrap())
        .middleware(Logging)
        .middleware(Redirection::new("/", "/login"))
        .get("/*", ServeDir::new("/", "examples/todo_app/public"))
        .scope("/api", api_routes())
        .scope("/", page_routes());

    Server::new(addr)
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .start(app)
}
