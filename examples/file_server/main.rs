use http1::server::Server;
use http1_web::{app::App, fs::ServeDir};

fn main() -> std::io::Result<()> {
    let app = App::new().get(
        "/*",
        ServeDir::new("examples/file_server/static")
            .resolve_index_html()
            .list_directory(true),
    );

    Server::new()
        .on_ready(|addr| println!("Listening on: http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}
