use http1::{
    body::ChunkedBody,
    http::{response::Response, status::StatusCode},
    server::Server,
};
use std::{thread, time::Duration};

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    server
        .listen(|req| {
            println!("Request: {req:?}");
            let (chunked, sender) = ChunkedBody::new();

            sender.send(b"<h1>Hello</h1>".to_vec()).ok();

            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                sender.send(b"<h2>World</h2>".to_vec()).ok();
            });

            Response::builder()
                .insert_header("X-Server", "MyServer")
                .insert_header("Content-Type", "text/html")
                .insert_header("Transfer-Encoding", "chunked")
                .insert_header("Connection", "keep-alive")
                .status(StatusCode::OK)
                .build(chunked)
        })
        .unwrap();
}
