use http1::{body::Body, response::Response, server::Server, status::StatusCode};

fn main() -> std::io::Result<()> {
    let server = Server::new();
    server
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", |_| {
            Response::new(StatusCode::OK, Body::new("Hello World!"))
        })
}
