use std::collections::HashMap;

use http1::server::Server;
use http1_web::{
    app::App,
    forms::form::Form,
    html::Html,
    middleware::sessions::{provider::SessionProvider, session::Session, store::MemoryStore},
    redirect::Redirect,
    IntoResponse,
};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);
    let app = App::new()
        .middleware(SessionProvider::new(MemoryStore::new()))
        .get("/", index)
        .post("/increment", increment)
        .post("/decrement", decrement)
        .post("/update_username", update_username);

    Server::new()
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:5000", app)
}

fn index(session: Session) -> impl IntoResponse {
    let count = session.get::<i64>("count").unwrap().unwrap_or_default();
    let username = session
        .get::<String>("username")
        .unwrap()
        .unwrap_or_else(|| String::from("<i>unknown</i>"));

    Html(format!(
        r#"
       <html>
           <head>
               <meta charset="utf-8"/>
               <title>App Sessions 📑</title>
           </head>
           <body>
               <h1>User session</h1>
               <h3>User: {username}</h3>
               <hr/>

               <form method="post" action="/update_username">
                    <input name="username" placeholder="Username..." required minlength="2"/>
                    <button>Update</button>
               </form>
   
               <form method="post">
                    <input type="submit" formaction="/increment" value="Increment"/>
                    <span>{count}</span>
                    <input type="submit" formaction="/decrement" value="Decrement"/>
               </form>
           </body>
       </html>
       "#
    ))
}

fn increment(mut session: Session) -> impl IntoResponse {
    session.update_or_insert("count", 1_i64, |x| x + 1).unwrap();
    Redirect::see_other("/")
}

fn decrement(mut session: Session) -> impl IntoResponse {
    session.update_or_insert("count", 1_i64, |x| x - 1).unwrap();
    Redirect::see_other("/")
}

fn update_username(
    mut session: Session,
    Form(mut form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(username) = form.remove("username") {
        session.insert("username", username).unwrap();
    }

    Redirect::see_other("/")
}
