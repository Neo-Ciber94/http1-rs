use http1::{http::Response, server::Server};

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    server
        .listen(|req| {
            println!("Request: {req:?}");
            Response::builder()
                .insert_header("X-Server", "Custom")
                .build(format!("Hello World"))
        })
        .unwrap();
}
