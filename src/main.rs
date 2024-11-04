use http1::server::Server;
use http1_web::app::App;

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let server = Server::new("localhost:5678");

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start(App::new().get("/", || "Hello World!"))
}
