use http1::{
    body::Body, headers, request::Request, response::Response, runtime::ThreadPooledRuntime,
    server::Server, status::StatusCode,
};
use http1_web::{app::App, handler::BoxedHandler};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    let app = App::new()
        .on_request(middleware)
        .get("/hello", || Response::new(StatusCode::OK, "Hello World!"))
        .get("/bocchi", || {
            let cwd = std::env::current_dir().unwrap();
            let file_dir = cwd.join("assets/bocchi.jpg");
            let file = File::open(file_dir).unwrap();

            Response::builder()
                .insert_header("X-Server", "MyServer")
                .insert_header(headers::CONTENT_TYPE, "image/jpg")
                .status(StatusCode::OK)
                .build(Body::new(file))
        });

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start_with(ThreadPooledRuntime::new()?, app)
}

fn middleware(req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
    println!("Request: {req:#?}");
    next.call(req)
}
