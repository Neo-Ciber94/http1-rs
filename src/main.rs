use std::time::Duration;

use http1::{body::Body, response::Response, server::Server};
use http1_web::{app::App, ws::WebSocketUpgrade, ErrorResponse};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let server = Server::new("localhost:5678");

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start(
            App::new()
                .get("/", || "Hello World!")
                .get("/ws", web_socket),
        )
}

fn web_socket(upgrade: WebSocketUpgrade) -> Result<Response<Body>, ErrorResponse> {
    let (mut ws, res) = upgrade.upgrade()?;

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1000));
        let msg = ws.read().unwrap();
        println!("Message: {msg:?}");
    });

    Ok(res)
}
