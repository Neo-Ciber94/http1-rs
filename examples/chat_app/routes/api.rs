use app::db::KeyValueDatabase;
use datetime::DateTime;
use http1::{
    body::Body,
    common::{broadcast::Broadcast, thread_pool::ThreadPool, uuid::Uuid},
    response::Response,
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
use serde::impl_serde_struct;
use std::{
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use crate::models::{ChatClient, ChatMessage, ChatRoom, ChatUser};

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
    State(db): State<KeyValueDatabase>,
    user: ChatUser,
    State(broadcast): State<Broadcast<ChatMessage>>,
    upgrade: WebSocketUpgrade,
) -> Response<Body> {
    let (pending, res) = upgrade.upgrade();

    std::thread::spawn(move || {
        let mut websocket = pending.wait().expect("failed to upgrade websocket");
        {
            let message_json = serde::json::to_string(&ChatMessage {
                id: Uuid::new_v4(),
                content: format!("{} joined", user.username),
                username: String::from("server"),
            })
            .unwrap();
            websocket.send(message_json).unwrap();
        }

        let ws = Arc::new(Mutex::new(websocket));
        let (tx, rx) = std::sync::mpsc::channel();

        let broadcast_thread = {
            let ws = Arc::clone(&ws);
            std::thread::spawn(move || {
                for msg in rx.iter() {
                    log::info!("Broadcasting message: {:?}", msg);
                    let mut lock = ws.lock().expect("Failed to lock WebSocket");
                    let message_json = serde::json::to_string(&msg).unwrap();
                    if let Err(e) = lock.send(message_json) {
                        log::error!("Failed to send message: {:?}", e);
                    }
                }
            })
        };

        // Wait for messages
        // let _subscription = {
        //     let ws = Arc::clone(&ws);
        //     let user = user.clone();

        //     broadcast
        //         .subscribe(move |msg| {
        //             log::info!("receiving message");
        //             if user.username == msg.username {
        //                 return;
        //             }

        //             let mut lock = ws.lock().expect("failed to get ws lock");
        //             let message_json = serde::json::to_string(&msg).unwrap();
        //             log::info!("broadcasting message from user: {}", user.username);
        //             lock.send(message_json).expect("failed to send message");
        //         })
        //         .unwrap()
        // };

        let _subscription = {
            let user = user.clone();

            broadcast
                .subscribe(move |msg| {
                    log::info!("receiving message");
                    if user.username == msg.username {
                        return;
                    }

                    tx.send(msg).unwrap();
                })
                .unwrap()
        };

        // Read incoming messages
        let t = std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(100));
                let mut lock = ws.lock().expect("failed to get ws lock");
                let next_msg = lock.recv().expect("failed to read message");
                drop(lock);

                if let Message::Text(content) = next_msg {
                    let id = Uuid::new_v4();
                    let content = content.trim().to_owned();
                    let username = user.username.clone();

                    // Save to the db
                    let message = ChatMessage {
                        id,
                        content,
                        username,
                    };

                    log::info!("new message: {message:?}");
                    db.set(format!("message/{}", message.id), message.clone())
                        .expect("failed to save value");

                    broadcast
                        .send(message)
                        .expect("failed to broadcast message");
                }
            }
        });

        t.join().unwrap();
    });

    res
}
