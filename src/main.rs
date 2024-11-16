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
    let (pending, res) = upgrade.upgrade()?;

    std::thread::spawn(move || {
        let mut ws = pending.wait().unwrap();
        std::thread::sleep(Duration::from_millis(5000));
        let msg = ws.recv().unwrap();
        let text = msg.as_str();
        println!("Message: {text:?}");

        ws.send("message").unwrap();

        std::thread::sleep(Duration::from_secs(2));
        ws.close().unwrap();
    });

    Ok(res)
}
