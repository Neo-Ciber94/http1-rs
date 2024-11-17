use app::db::KeyValueDatabase;
use datetime::DateTime;
use http1::{
    body::Body,
    common::{thread_pool::ThreadPool, uuid::Uuid},
    response::Response,
};
use http1_web::{
    app::Scope, cookies::Cookie, forms::form::Form, json::Json, state::State, ws::WebSocketUpgrade,
    ErrorResponse, HttpResponse,
};
use serde::impl_serde_struct;
use std::sync::LazyLock;

use crate::models::{ChatMessage, ChatRoom, ChatUser};

static THREAD_POOL: LazyLock<ThreadPool> = LazyLock::new(|| {
    ThreadPool::builder()
        .spawn_on_full(true)
        .build()
        .expect("failed to create thread pool")
});

pub fn routes() -> Scope {
    Scope::new()
        .post("/login", login)
        .get("/me", me)
        .get("/logout", logout)
        .get("/chat", chat)
        .get("/chat/messages", messages)
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
    _user: ChatUser,
) -> Result<Json<Vec<ChatMessage>>, ErrorResponse> {
    let messages = db.scan("message/")?;
    Ok(Json(messages))
}

fn chat(
    state: State<KeyValueDatabase>,
    user: ChatUser,
    State(chat_room): State<ChatRoom>,
    upgrade: WebSocketUpgrade,
) -> Response<Body> {
    let (pending, res) = upgrade.upgrade();

    THREAD_POOL
        .execute(move || {
            let ws = pending
                .wait()
                .expect("failed to complete websocket connection");

            // handle chat
            chat_conversation(state, user.clone(), chat_room.clone());

            // save chat to track it
            let ChatRoom(chat_room) = chat_room;
            let mut lock = chat_room.lock().unwrap();
            lock.push((ws, user));
        })
        .unwrap();

    res
}

fn chat_conversation(
    State(db): State<KeyValueDatabase>,
    user: ChatUser,
    ChatRoom(chat_room): ChatRoom,
) {
    log::info!("Starting chat: {user:?}");

    std::thread::spawn(move || loop {
        let mut lock = chat_room.lock().unwrap();
        let (ws, _) = lock
            .iter_mut()
            .find(|(_, u)| u.username == user.username)
            .unwrap();

        let msg = ws.recv().unwrap();
        drop(lock);

        if let Some(content) = msg.into_text() {
            let id = Uuid::new_v4();
            let username = user.username.clone();

            // Save to the db
            let message = ChatMessage {
                id,
                username,
                content,
            };

            log::info!("new message: {message:?}");
            db.set(format!("message/{}", message.id), message.clone())
                .unwrap();

            // Broadcast to all the users
            let mut lock = chat_room.lock().unwrap();
            let other_users = lock.iter_mut().filter(|(_, u)| u.username != user.username);

            let message_json = serde::json::to_string(&message).unwrap();
            for (ws, _) in other_users {
                ws.send(message_json.as_str()).unwrap();
            }
        }
    });
}
