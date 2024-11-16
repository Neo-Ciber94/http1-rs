use http1::{body::Body, response::Response, server::Server};
use http1_web::{
    app::App,
    ws::{Message, WebSocketUpgrade},
    ErrorResponse,
};

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
    let (pending, res) = upgrade.upgrade();

    std::thread::spawn(move || {
        let mut ws = pending.wait().unwrap();

        std::thread::spawn(move || {
            while let Ok(msg) = ws.recv() {
                if let Message::Text(text) = msg {
                    println!("{text}")
                }
            }
        });
    });

    Ok(res)
}
