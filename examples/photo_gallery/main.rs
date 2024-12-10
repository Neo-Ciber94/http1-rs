use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    str::FromStr,
};

use app::db::KeyValueDatabase;
use http1::{common::uuid::Uuid, server::Server};
use http1_web::{
    app::App,
    forms::multipart::{FormEntry, Multipart},
    fs::ServeDir,
    html::{self, element::HTMLElement},
    middleware::logging::Logging,
    mime::Mime,
    query::Query,
    redirect::Redirect,
    state::State,
    ErrorResponse,
};

const MAX_BODY_SIZE: usize = 1024 * 1024 * 1024; // 1 GB

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
        .state(KeyValueDatabase::new("examples/photo_gallery/db.json").unwrap())
        .get(
            "/static/*",
            ServeDir::new("examples/photo_gallery/static").use_cache_headers(true),
        )
        .get("/*", ServeDir::new("examples/photo_gallery/public"))
        .get("/", gallery_page)
        .get("/upload", upload_page)
        .post("/api/upload", upload);

    Server::new()
        .include_conn_info(true)
        .max_body_size(Some(MAX_BODY_SIZE))
        .on_ready(|addr| log::info!("Listening on: http://{addr}"))
        .listen("localhost:5000", app)
}

fn gallery_page(
    State(db): State<KeyValueDatabase>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<HTMLElement, ErrorResponse> {
    let images = db.scan::<Image>("image/")?;
    let preview_img = query.get("preview");

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

            if let Some(preview_image_id) = preview_img {
                if let Some(img) = images.iter().find(|x| x.id.as_str() == preview_image_id) {
                    components::image_preview(&img.image_url);
                }
            }

            html::a(|| {
                html::class("gallery-link");
                html::attr("href", "/upload");
                html::content("Upload Image");
            });

            html::div(|| {
                html::class("gallery-container");

                if images.is_empty() {
                    html::h2("No images");
                }

                html::div(|| {
                    html::class("gallery");

                    images.iter().for_each(|img| {
                        html::a(|| {
                            html::class("gallery-image");
                            html::attr("href", format!("?preview={}", img.id));
                            html::img(|| {
                                html::attr("src", format!("/static{}", img.image_url));
                                html::attr("height", "400px");
                                html::attr("width", "400px");
                            });
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

mod components {
    use http1_web::html;

    pub fn image_preview(src: &str) {
        html::a(|| {
            html::class("image-preview");
            html::attr("href", "/");

            html::div(|| {
                html::attr("data-close-btn", true);
                html::content("✖");
            });
        });

        html::img(|| {
            html::class("image-preview-img");
            html::attr("src", format!("/static{src}"));
        });
    }
}

#[derive(Debug)]
struct UploadFile {
    image: FormEntry,
}

serde::impl_deserialize_struct!(UploadFile => {
    image: FormEntry
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
