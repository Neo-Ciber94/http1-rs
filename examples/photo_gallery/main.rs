use app::db::KeyValueDatabase;
use http1::server::Server;
use http1_web::{
    app::App,
    fs::ServeDir,
    html::{self, element::HTMLElement},
};

pub fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let app = App::new()
        .state(KeyValueDatabase::new("examples/photo_gallery/db.json").unwrap())
        .get(
            "/static/*",
            ServeDir::new("/static", "examples/photo_gallery/static"),
        )
        .get("/*", ServeDir::new("/", "examples/photo_gallery/public"))
        .get("/", gallery);

    Server::new("http://localhost:5000")
        .on_ready(|addr| {
            log::info!("Listening on: {addr}");
        })
        .start(app)
}

fn gallery() -> HTMLElement {
    html::html(|| {
        html::attr("lang", "en");
        html::head(|| {
            html::title("Photo Gallery");

            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::link(|| {
                html::attr("rel", "stylesheet");
                html::attr("href", "/styles.css");
            });
        });

        html::body(|| {
            html::h2("Gallery");

            html::a(|| {
                html::attr("href", "/upload");
            });

            html::div(|| {});
        });
    })
}
