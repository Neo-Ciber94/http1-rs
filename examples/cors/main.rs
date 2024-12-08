use std::sync::{Arc, Mutex};

use http1::{
    body::Body,
    headers::{self, HeaderName, HeaderValue},
    method::Method,
    server::Server,
};
use http1_web::{
    app::App,
    forms::form::Form,
    handler::BoxedHandler,
    header::{GetHeader, Referer},
    html::Html,
    json::Json,
    middleware::{cors::Cors, logging::Logging},
    redirect::Redirect,
    state::State,
    HttpResponse, IntoHttpResponse,
};
use serde::impl_serde_struct;

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    // let t1 = std::thread::spawn(move || backend().expect("failed to start backend"));
    // let t2 = std::thread::spawn(move || frontend().expect("failed to start frontend"));

    // while !t1.is_finished() || !t2.is_finished() {}

    backend().expect("failed to start backend");
    Ok(())
}

#[derive(Debug, Clone)]
struct Flower {
    name: String,
    color: String,
}

impl_serde_struct!(Flower => {
    name: String,
    color: String,
});

#[derive(Default, Clone, Debug)]
struct AppState {
    flowers: Arc<Mutex<Vec<Flower>>>,
}

fn backend() -> std::io::Result<()> {
    let app = App::new()
        .middleware(
            Cors::builder()
                .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                .allow_origins(vec!["http://localhost:3456"])
                .allow_any_header()
                .build(),
        )
        .middleware(Logging)
        // .state(AppState::default())
        // .get("/api/flowers", |State(state): State<AppState>| {
        //     let lock = state.flowers.lock().unwrap();
        //     let flowers = (*lock).clone();
        //     Json(flowers).into_http_response().append_header(
        //         headers::ACCESS_CONTROL_ALLOW_ORIGIN,
        //         HeaderValue::from_static("http://localhost:3456"),
        //     )
        // })
        .get("/api/flowers", || Json(Vec::<Flower>::new()))
        .post("/api/flowers", |flower: String| {
            dbg!(&flower);
            // let mut lock = state.flowers.lock().unwrap();
            // lock.push(flower);

            // match referer {
            //     Some(GetHeader(referer)) => {
            //         let s = referer.to_string();
            //         println!("{s}");
            //         Redirect::see_other(referer.to_string()).into_http_response()
            //     }
            //     None => HttpResponse::created(Body::empty()),
            // }
            "Hello World"
        });

    Server::new()
        .on_ready(|addr| println!("Backend listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:5000", app)
}

fn frontend() -> std::io::Result<()> {
    let app = App::new().get("/", || {
        Html(
            r#"
        <html>
            <head>
                <title>🌹🪻 Flowers 🌻🌷</title>
                <meta charset="utf-8">
                <style>
                    .flower-color {
                        display: inline-block;
                        margin-right: 8px;
                        width: 16px;
                        height: 16px;                        
                    }
                </style>
            </head>
            <body>
                <h1>🌸🌸 Flowers 🌸🌸</h1>
                <div id="flower-list">
                   Loading...
                </div>

                <h2>New Flower</h2>
                <form method="post" action="http://localhost:5000/api/flowers">
                    <input name="name" type="text" required />
                    <input name="color" type="color" required />
                    <br/>
                    <button>Add Flower</button>
                </form>
                <script>
                    window.onload = () => {
                        const flowersEl = document.getElementById('flower-list');
                        
                        fetch('http://localhost:5000/api/flowers')
                            .then(res => res.json()) 
                            .then(flowers => {
                                console.log({ flowers })
                                if (flowers.length === 0) {
                                   flowersEl.textContent = 'No flowers available 🥀'; 
                                }
                                else {
                                    const listEl = document.createElement("ul");

                                    for (const flower of flowers) {
                                        const flowerEl = document.createElement("li");
                                        const colorEl = document.createElement("span");
                                        const textEl = document.createElement("span");

                                        colorEl.classList.add("flower-color");
                                        colorEl.style.backgroundColor = flower.color;
                                        textEl.textContent = flower.name;

                                        flowerEl.appendChild(colorEl);
                                        flowerEl.appendChild(textEl);
                                        listEl.appendChild(flowerEl);
                                    }

                                    flowersEl.replaceChildren(listEl);
                                }
                            })
                            .catch(error => {
                                console.error('Error fetching users:', error);
                                flowersEl.textContent = '❌ Failed to load flowers'; 
                            });
                    };
                </script>
            </body>
        </html>
        "#,
        )
    });

    Server::new()
        .on_ready(|addr| println!("Frontend listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3456", app)
}
