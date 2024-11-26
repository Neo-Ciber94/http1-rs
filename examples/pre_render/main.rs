use http1::server::Server;
use http1_web::{
    app::{App, PreRenderConfig},
    fs::ServeDir,
    html,
};

fn main() {
    log::set_logger(log::ConsoleLogger);

    http1_web::app::pre_render(
        app(),
        "examples/pre_render/static/pages",
        PreRenderConfig::default(),
    )
    .expect("failed to pre render");

    Server::new()
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .listen("localhost:4000", app())
        .expect("failed to listen");
}

fn app() -> App {
    App::new()
        .get("/*", ServeDir::new("examples/pre_render/static/pages"))
        .get("/", || {
            html::html(|| {
                html::body(|| {
                    html::a(|| {
                        html::attr("href", "/a");
                        html::content("Page A");
                    });

                    html::a(|| {
                        html::attr("href", "/b");
                        html::content("Page B");
                    });

                    html::a(|| {
                        html::attr("href", "/c");
                        html::content("Page C");
                    });
                });
            })
        })
        .get("/a", || {
            html::html(|| {
                html::body(|| {
                    html::h1("Page A");
                });
            })
        })
        .get("/b", || {
            html::html(|| {
                html::body(|| {
                    html::h1("Page B");
                });
            })
        })
        .get("/c", || {
            html::html(|| {
                html::body(|| {
                    html::h1("Page C");
                });
            })
        })
}
