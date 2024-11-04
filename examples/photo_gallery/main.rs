use std::{
    fs::File,
    io::{BufReader, BufWriter},
    str::FromStr,
};

use app::db::KeyValueDatabase;
use http1::{common::uuid::Uuid, server::Server};
use http1_web::{
    app::App,
    forms::{
        form_data::FormDataConfig,
        multipart::{FormFile, Multipart},
    },
    fs::ServeDir,
    html::{self, element::HTMLElement},
    middleware::logging::Logging,
    mime::Mime,
    redirect::Redirect,
    state::State,
    ErrorResponse,
};

const MAX_BODY_SIZE: usize = 1024 * 1024 * 1024;

#[derive(Debug)]
struct Image {
    id: String,
    image_url: String,
}

serde::impl_serde_struct!(Image => {
    id: String,
    image_url: String
});

pub fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let app = App::new()
        .middleware(Logging)
        .state(FormDataConfig::default().max_body_size(MAX_BODY_SIZE))
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
        .max_body_size(Some(MAX_BODY_SIZE))
        .on_ready(|addr| log::info!("Listening on: http://{addr}"))
        .start(app)
}

fn gallery_page(State(db): State<KeyValueDatabase>) -> Result<HTMLElement, ErrorResponse> {
    let images = db.scan::<Image>("image/")?;

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
            html::header(|| {
                html::a(|| {
                    html::attr("href", "/");
                    html::content("Photo Gallery");
                });
            });

            html::h2("Gallery");

            html::a(|| {
                html::class("gallery-link");
                html::attr("href", "/upload");
                html::content("Upload Image");
            });

            html::div(|| {
                if images.is_empty() {
                    html::h2("No images");
                }

                html::div(|| {
                    html::class("gallery");

                    images.iter().for_each(|img| {
                        html::img(|| {
                            html::attr("src", format!("/static{}", img.image_url));
                        });
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
            html::header(|| {
                html::a(|| {
                    html::attr("href", "/");
                    html::content("Photo Gallery");
                });
            });

            html::h2("Gallery | Upload Image");

            html::form(|| {
                html::attr("action", "/api/upload");
                html::attr("method", "post");
                html::attr("enctype", "multipart/form-data");

                html::input(|| {
                    html::attr("accept", "image/*");
                    html::attr("type", "file");
                    html::attr("name", "image");
                    html::attr("required", true);
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
    Multipart(form): Multipart<UploadFile>,
) -> Result<Redirect, ErrorResponse> {
    let mime = Mime::from_str(form.image.content_type().unwrap())?;

    if !Mime::ANY_IMAGE.matches(&mime) {
        return Err(ErrorResponse::from_error(
            http1_web::ErrorStatusCode::BadRequest,
            "expected image",
        ));
    }

    let cwd = std::env::current_dir().expect("failed to get current directory");
    let dir = cwd.join("examples/photo_gallery/static");
    let filename = Uuid::new_v4().to_simple_string();

    let ext = mime.extension_with_sep().unwrap_or(".bin");
    let file_path = dir.join(format!("{filename}{ext}",));

    log::debug!("Writing image to: {file_path:?}");

    let source = form.image.file().read(true).open().unwrap();
    let dest_file = File::create(&file_path)?;
    let mut reader = BufReader::new(source);
    let mut writer = BufWriter::new(dest_file);
    std::io::copy(&mut reader, &mut writer)?;

    // Save image on db
    let id = Uuid::new_v4().to_simple_string();
    let image_url = format!("/{filename}{ext}");

    match db.set(
        format!("image/{id}"),
        Image {
            id: id.clone(),
            image_url: image_url.clone(),
        },
    ) {
        Ok(_) => {
            log::info!("Image saved {id} => {image_url}")
        }
        Err(err) => {
            log::error!("Failed to save image: {err}");

            // Remove file if fails to save
            std::fs::remove_file(file_path)?;
        }
    }

    Ok(Redirect::see_other("/"))
}
