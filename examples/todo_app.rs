use http1::server::Server;
use http1_web::{app::App, into_response::IntoResponse};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "127.0.0.1:5000".parse().unwrap();

    Server::new(addr)
        .on_ready(|addr| log::debug!("Listening on http://{addr}"))
        .start(App::new().get("/", hello))
}

fn hello() -> impl IntoResponse {
    "Hello World!"
}
