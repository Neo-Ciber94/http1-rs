use app::db::KeyValueDatabase;
use datetime::DateTime;
use http1::common::uuid::Uuid;
use http1::{
    body::Body, common::broadcast::Broadcast, request::Request, response::Response, server::Server,
};
use http1_web::ws::PendingWebSocketUpgrade;
use http1_web::{
    app::App, fs::ServeDir, handler::BoxedHandler, middleware::logging::Logging,
    redirect::Redirect, IntoResponse, RequestExt,
};
use http1_web::{
    app::Scope,
    cookies::Cookie,
    forms::form::Form,
    json::Json,
    state::State,
    ws::{Message, WebSocketUpgrade},
    ErrorResponse, HttpResponse,
};
use models::{ChatMessage, ChatUser};
use serde::impl_serde_struct;

mod constants;
mod models;

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let server = Server::new();
    let broadcast = Broadcast::<ChatMessage>::new();

    let app = App::new()
        .middleware(Logging)
        .middleware(auth_middleware)
        .state(KeyValueDatabase::new("examples/chat_app/db.json").unwrap())
        .state(broadcast)
        .scope(
            "/api",
            Scope::new()
                .post("/login", login)
                .get("/me", me)
                .get("/logout", logout)
                .get("/chat", chat)
                .get("/chat/messages", messages),
        )
        .get(
            "/*",
            ServeDir::new("examples/chat_app/static").resolve_index_html(),
        );

    server
        .on_ready(|addr| log::debug!("Listening on http://localhost:{}", addr.port()))
        .listen("localhost:5678", app)
}

fn auth_middleware(mut req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
    // Pages to check for auth
    const PAGES: &[&str] = &["/login", "/"];

    let path = req.uri().path_and_query().path();

    if !PAGES.contains(&path) {
        return next.call(req);
    }

    let to_login = path == "/login";
    let is_authenticated = http1_web::guard!(req.extract::<Option<ChatUser>>()).is_some();

    if is_authenticated && to_login {
        Redirect::see_other("/").into_response()
    } else if !is_authenticated && !to_login {
        Redirect::see_other("/login").into_response()
    } else {
        next.call(req)
    }
}

struct LoginInput {
    username: String,
}

impl_serde_struct!(LoginInput => {
    username: String
});

fn login(Form(input): Form<LoginInput>) -> Result<HttpResponse, ErrorResponse> {
    let session_id = http1::common::base64::encode(input.username);
    let cookie = Cookie::new(crate::constants::COOKIE_AUTH_SESSION, session_id)
        .path("/")
        .http_only(true)
        .max_age(crate::constants::SESSION_DURATION_SECS);

    let res = HttpResponse::see_other("/").set_cookie(cookie).finish();

    Ok(res)
}

fn logout(_user: ChatUser) -> HttpResponse {
    let cookie = Cookie::new(crate::constants::COOKIE_AUTH_SESSION, "")
        .path("/")
        .expires(DateTime::UNIX_EPOCH);

    HttpResponse::see_other("/login")
        .set_cookie(cookie)
        .finish()
}

fn me(user: ChatUser) -> Json<ChatUser> {
    Json(user)
}

fn messages(
    State(db): State<KeyValueDatabase>,
    _user: ChatUser, // The user only is extracted if the user is logged, otherwise return a 401
) -> Result<Json<Vec<ChatMessage>>, ErrorResponse> {
    let messages = db.scan("message/")?;
    Ok(Json(messages))
}

fn chat(
    State(db): State<KeyValueDatabase>,
    user: ChatUser,
    State(broadcast): State<Broadcast<ChatMessage>>,
    upgrade: WebSocketUpgrade,
) -> Response<Body> {
    let (pending, response) = upgrade.upgrade();

    // Start chat in other thread
    std::thread::spawn(move || join_chat(pending, user, broadcast, db));

    // We send the response to upgrade the connection to use websocket
    response
}

fn join_chat(
    pending: PendingWebSocketUpgrade,
    user: ChatUser,
    broadcast: Broadcast<ChatMessage>,
    db: KeyValueDatabase,
) {
    // Wait until the connection is upgraded
    let websocket = pending.wait().expect("failed to upgrade websocket");
    let (mut tx, mut rx) = websocket.split().unwrap();

    // Notify a new user is on the chat
    broadcast
        .send(ChatMessage {
            id: Uuid::new_v4(),
            content: format!("`{}` joined", user.username),
            username: String::from("server"),
        })
        .unwrap();

    // Wait for messages
    let _subscription = {
        let user = user.clone();

        broadcast
            .subscribe(move |msg| {
                log::info!("receiving message");

                // Prevent we send user own message to itself
                if user.username == msg.username {
                    return;
                }

                let message_json = serde::json::to_string(&msg).unwrap();
                log::info!("broadcasting message from user: {}", user.username);
                tx.send(message_json).expect("failed to send message");
            })
            .unwrap()
    };

    // Read incoming messages
    let t = std::thread::spawn(move || {
        loop {
            let next_msg = rx.recv().expect("failed to read message");

            if let Message::Text(content) = next_msg {
                let id = Uuid::new_v4();
                let content = content.trim().to_owned();
                let username = user.username.clone();

                let message = ChatMessage {
                    id,
                    content,
                    username,
                };

                log::info!("new message: {message:?}");

                // Save to the db
                db.set(format!("message/{}", message.id), message.clone())
                    .expect("failed to save value");

                // Send to all users
                broadcast
                    .send(message)
                    .expect("failed to broadcast message");
            }
        }
    });

    t.join().unwrap();
}
