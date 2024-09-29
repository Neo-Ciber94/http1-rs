use http1::{
    body::Body, headers, request::Request, response::Response, runtime::ThreadPooledRuntime,
    server::Server, status::StatusCode,
};
use http1_web::app::App;
use std::fs::File;

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    let app = App::new().get("/hello", |()| Response::new(StatusCode::OK, "Hello World!"));

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start_with(ThreadPooledRuntime::new().unwrap(), app)
        .unwrap();
}

fn handle_request(req: Request<Body>) -> Response<Body> {
    println!("Request: {req:#?}");

    let cwd = std::env::current_dir().unwrap();
    let file_dir = cwd.join("assets/bocchi.jpg");
    let file = File::open(file_dir).unwrap();

    Response::builder()
        .insert_header("X-Server", "MyServer")
        .insert_header(headers::CONTENT_TYPE, "image/jpg")
        .status(StatusCode::OK)
        .build(Body::new(file))
}
