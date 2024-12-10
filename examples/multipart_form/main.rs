use std::collections::HashMap;

use http1::server::Server;
use http1_web::{
    app::App,
    forms::multipart::{FormEntry, Multipart},
    html::Html,
    redirect::Redirect,
    IntoResponse,
};

fn main() -> std::io::Result<()> {
    let app = App::new().get("/", index_html).post("/", accept_multipart);

    Server::new()
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}

fn index_html() -> impl IntoResponse {
    Html(
        r#"
        <html>
            <head>
                <title>Multipart Form 📂</title>
                <meta charset="utf-8"/>
            </head>
            <body>
                <form method="post" action="/" enctype="multipart/form-data">
                    <label>
                        Upload files
                        <input type="file" name="file" multiple required />
                    </label>
                    <hr/>
                    <button>Upload</button>
                </form>
            </body>
        </html>
    "#,
    )
}

fn accept_multipart(
    Multipart(form): Multipart<HashMap<String, Vec<FormEntry>>>,
) -> impl IntoResponse {
    for (name, files) in form {
        for file in files {
            let content_type = file.content_type().unwrap().to_owned();
            let file_name = file.filename().unwrap().to_owned();
            let length_bytes = file.bytes().unwrap().len();

            println!("Size of `{name}` with file name `{file_name}` with type `{content_type}` is {length_bytes} bytes")
        }
    }

    Redirect::see_other("/")
}
