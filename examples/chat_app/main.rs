use http1::{common::uuid::Uuid, server::Server};
use http1_web::app::App;
use serde::impl_serde_struct;

#[derive(Debug)]
pub struct ChatMessage {
    pub id: Uuid,
    pub user_id: String,
    pub content: String,
}

impl_serde_struct!(ChatMessage => {
     id: Uuid,
     user_id: String,
     content: String,
});

#[derive(Debug)]
pub struct ChatUser {
    id: Uuid,
    username: String,
}

impl_serde_struct!(ChatUser => {
     id: Uuid,
     username: String
});

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let server = Server::new("localhost:5678");

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start(App::new().get("/", || "Hello World!"))
}
