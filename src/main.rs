use http1::{
    body::{Body, ChunkedBody},
    headers,
    request::Request,
    response::Response,
    runtime::SingleThreadRuntime,
    server::Server,
    status::StatusCode,
};
use std::{thread, time::Duration};

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start_with(SingleThreadRuntime, handle_request)
        .unwrap();
}

fn handle_request(req: Request<Body>) -> Response<Body> {
    println!("Request: {req:#?}");
    let (chunked, sender) = ChunkedBody::new();

    sender.send("<h1>Hello</h1>").ok();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        sender.send("<h2>World</h2>").ok();
    });

    Response::builder()
        .insert_header("X-Server", "MyServer")
        .insert_header(headers::CONTENT_TYPE, "text/html")
        .insert_header(headers::TRANSFER_ENCODING, "chunked")
        .insert_header(headers::CONNECTION, "keep-alive")
        .status(StatusCode::OK)
        .build(chunked.into())
}
