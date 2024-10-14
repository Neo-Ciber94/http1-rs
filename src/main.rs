use http1::{
    body::Body, headers, request::Request, response::Response, runtime::ThreadPooledRuntime,
    server::Server, status::StatusCode, uri::uri::Uri,
};
use http1_web::{
    app::App, forms::{
        multipart::{FormFile, Multipart},
    }, handler::BoxedHandler, html, impl_deserialize_struct, json::Json, path::Path, serde::json::value::JsonValue
};
use std::{collections::HashMap, fs::File};

#[derive(Debug)]
struct Upload {
    string: String,
    file: FormFile,
}

impl_deserialize_struct!(Upload => {
    string: String,
    file: FormFile
});

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let server = Server::new(addr);

    let app = App::new()
        .middleware(middleware)
        .get("/hello", || Response::new(StatusCode::OK, "Hello World!"))
        .get("/bocchi", |_uri: Uri, _body: Body| {
            let cwd = std::env::current_dir().unwrap();
            let file_dir = cwd.join("assets/bocchi.jpg");
            let file = File::open(file_dir).unwrap();

            Response::builder()
                .insert_header("X-Server", "MyServer")
                .insert_header(headers::CONTENT_TYPE, "image/jpg")
                .status(StatusCode::OK)
                .body(Body::new(file))
        })
        .get("/", || {
            html::html(|| {
                html::attr("lang", "en");
                html::head(|| {
                    html::title("My Simple Webpage");
                    html::style(
                        r#"
                            body {
                                font-family: Arial, sans-serif;
                                line-height: 1.6;
                                color: #333;
                                max-width: 800px;
                                margin: 0 auto;
                                padding: 20px;
                            }
                            h1 {
                                color: #2c3e50;
                                border-bottom: 2px solid #3498db;
                                padding-bottom: 10px;
                            }
                            p {
                                margin-bottom: 15px;
                            }
                            .highlight {
                                background-color: #f1c40f;
                                padding: 5px;
                                border-radius: 3px;
                            }
                        "#,
                    );
                });
                html::body(|| {
                    html::h1("Welcome to My Simple Webpage");
                    html::p("This is a basic webpage created using a custom HTML DSL in Rust.");
                    html::p(|| {
                        html::content("Here's some text with a ");
                        html::span(|| {
                            html::attr("class", "highlight");
                            html::content("highlighted portion");
                        });
                        html::content(".");
                    });
                    html::h2("Features of this page:");
                    html::ul(|| {
                        html::li("Simple and clean design");
                        html::li("Custom styling with CSS");
                        html::li("Responsive layout");
                    });
                });
            })
        })
        .post("/json", |json: Json<HashMap<String, JsonValue>>| {
            println!("{:?}", json);
            Json(http1_web::json!({
                hello: "world"
            }))
        })
        .get("/dynamic/:val", |path: Path<u32>| {
            Json(http1_web::json!({
               param: path.0
            }))
        })
        .post("/upload", |form: Multipart<Upload>| {
            println!("{form:?}");
        });

    server
        .on_ready(|addr| println!("Listening on http://{addr}"))
        .start_with(ThreadPooledRuntime::new()?, app)
}

fn middleware(req: Request<Body>, next: &BoxedHandler) -> Response<Body> {
    //println!("Request: {req:#?}");
    next.call(req)
}
