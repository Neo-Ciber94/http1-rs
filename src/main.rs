use http1::{
    body::Body,
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
            let (sender, body) = Body::stream();

            sender.send(b"Hello ".to_vec()).ok();

            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                sender.send(b"World".to_vec()).ok();
            });

            Response::builder()
                .insert_header("X-Server", "Custom")
                .insert_header("Content-Type", "text/html")
                .insert_header("Transfer-Encoding", "chunked")
                .insert_header("Connection", "keep-alive")
                .status(StatusCode::OK)
                .build(body)
        })
        .unwrap();
}
