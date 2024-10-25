use http1::server::Server;
use http1_web::app::App;

pub fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    Server::new("http://localhost:5000")
        .on_ready(|addr| {
            log::info!("Listening on: {addr}");
        })
        .start(App::new().get("/", || "Hello World"))
}
