use std::{
    fs::File,
    io::{BufReader, BufWriter},
    str::FromStr,
};

use app::db::KeyValueDatabase;
use http1::{common::uuid::Uuid, error::BoxError, server::Server};
use http1_web::{
    app::App,
    forms::{form_map::FormMap, multipart::{FormFile, Multipart}},
    fs::ServeDir,
    html::{self, element::HTMLElement},
    mime::Mime,
    redirect::Redirect,
    state::State,
    ErrorResponse,
};

struct Image {
    id: String,
}

serde::impl_serde_struct!(Image => {
    id: String,
});

pub fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let app = App::new()
        .state(KeyValueDatabase::new("examples/photo_gallery/db.json").unwrap())
        .get(
            "/static/*",
            ServeDir::new("/static", "examples/photo_gallery/static"),
        )
        .get("/*", ServeDir::new("/", "examples/photo_gallery/public"))
        .get("/", gallery_page)
        .get("/upload", upload_page)
        .post("/api/upload", upload);

    Server::new("localhost:5000")
        .on_ready(|addr| {
            log::info!("Listening on: http://{addr}");
        })
        .start(app)
}

fn gallery_page(State(db): State<KeyValueDatabase>) -> Result<HTMLElement, ErrorResponse> {
    let images = db.scan::<Image>("images/")?;

    Ok(html::html(|| {
        html::attr("lang", "en");
        html::head(|| {
            html::title("Photo Gallery");

            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::link(|| {
                html::attr("rel", "stylesheet");
                html::attr("href", "/styles.css");
            });
        });

        html::body(|| {
            html::h2("Gallery");

            html::a(|| {
                html::attr("href", "/upload");
                html::content("Upload Image");
            });

            html::div(|| {
                images.iter().for_each(|img| {
                    html::img(|| {
                        html::attr("href", format!("/static/{}", img.id));
                    });
                });
            });
        });
    }))
}

fn upload_page() -> HTMLElement {
    html::html(|| {
        html::attr("lang", "en");
        html::head(|| {
            html::title("Photo Gallery");

            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::link(|| {
                html::attr("rel", "stylesheet");
                html::attr("href", "/styles.css");
            });
        });

        html::body(|| {
            html::h2("Gallery | Upload Image");

            html::form(|| {
                html::attr("action", "/api/upload");
                html::attr("method", "post");
                html::attr("enctype", "multipart/form-data");

                html::input(|| {
                    // html::attr("accept", "image/*");
                    html::attr("type", "file");
                    html::attr("name", "image");
                });

                html::button(|| {
                    html::content("Upload");
                });
            });
        });
    })
}

#[derive(Debug)]
struct UploadFile {
    image: FormFile,
}

serde::impl_deserialize_struct!(UploadFile => {
    image: FormFile
});

fn upload(
    State(db): State<KeyValueDatabase>,
    Multipart(form): Multipart<UploadFile>
) -> Result<Redirect, ErrorResponse> {
    //dbg!(&form);

    let mime = Mime::from_str(form.image.content_type().unwrap())?;
    let cwd = std::env::current_dir().expect("failed to get current directory");
    let dir = cwd.join("examples/photo_gallery/static");
    let filename = Uuid::new_v4().to_simple_string();
    let file_path = dir.join(format!("{filename}.{}", mime.subtype()));

    log::debug!("Writing file to: {file_path:?}");

    let source = form.image.file().read(true).open().unwrap();
    let dest_file = File::create(file_path)?;
    let mut reader = BufReader::new(source);
    let mut writer = BufWriter::new(dest_file);
    std::io::copy(&mut reader, &mut writer)?;

    Ok(Redirect::see_other("/upload"))
}

// fn upload(
//     State(db): State<KeyValueDatabase>,
//     body: Vec<u8>,
// ) -> Result<Redirect, ErrorResponse> {
//     let cwd = std::env::current_dir().expect("failed to get current directory");
//     let dir = cwd.join("examples/photo_gallery/static");
//     let filename = Uuid::new_v4().to_simple_string();
//     let file_path = dir.join(format!("{filename}.jpg"));

//     log::debug!("Writing file to: {file_path:?}");

//     let dest_file = File::create(file_path)?;
//     let mut reader = BufReader::new(body.as_slice());
//     let mut writer = BufWriter::new(dest_file);
//     std::io::copy(&mut reader, &mut writer)?;

//     Ok(Redirect::see_other("/upload"))
// }

// fn upload(
//     State(db): State<KeyValueDatabase>,
//     mut form: FormMap,
// ) -> Result<Redirect, ErrorResponse> {
//     let data = form.remove("image").unwrap();
//     let cwd = std::env::current_dir().expect("failed to get current directory");
//     let dir = cwd.join("examples/photo_gallery/static");
//     let filename = Uuid::new_v4().to_simple_string();
//     let file_path = dir.join(format!("{filename}.jpg"));

//     log::debug!("Writing file to: {file_path:?}");

//     let dest_file = File::create(file_path)?;
//     let mut reader = BufReader::new(data.reader());
//     let mut writer = BufWriter::new(dest_file);
//     std::io::copy(&mut reader, &mut writer)?;

//     Ok(Redirect::see_other("/upload"))
// }
