use std::time::Duration;

use http1::{
    body::Body,
    response::{
        sse::{SseEvent, SseStream},
        Response,
    },
    server::Server,
};
use http1_web::{
    app::App, fs::ServeFile, middleware::logging::Logging, query::Query, IntoResponse,
};
use log::ConsoleLogger;
use serde::impl_serde_struct;

fn main() -> std::io::Result<()> {
    log::set_logger(ConsoleLogger);

    let app = App::new()
        .middleware(Logging)
        .get("/", ServeFile::new("examples/sse/index.html").unwrap())
        .get("/api/count", get_counter);

    Server::new()
        .on_ready(|addr| println!("listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}

#[derive(Default)]
struct Count {
    to: Option<u64>,
}

impl_serde_struct!(Count => {
    to: Option<u64>,
});

fn get_counter(query: Option<Query<Count>>) -> Response<Body> {
    let Query(count) = query.unwrap_or_default();
    let (tx, rx) = SseStream::new();

    std::thread::spawn(move || {
        let max = count.to.unwrap_or(u64::MAX);

        for i in 0..max {
            tx.send(SseEvent::with_data(i.to_string()).unwrap())
                .unwrap();

            std::thread::sleep(Duration::from_millis(1000));
        }

        tx.send(SseEvent::with_data("Done!").unwrap()).unwrap();
    });

    rx.into_response()
}
