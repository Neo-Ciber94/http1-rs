use http1::{runtime::ThreadPooledRuntime, server::Server};
use http1_web::app::App;

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start_with(ThreadPooledRuntime::new()?, App::new())
}
