use http1::server::Server;
use http1_web::{app::App, html, IntoResponse};

fn main() -> std::io::Result<()> {
    let app = App::new()
        .get("/", index_page)
        .get("/images", images_page)
        .get("/styled", styled_page);

    Server::new()
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}

fn index_page() -> impl IntoResponse {
    html::html(|| {
        html::head(|| {
            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::title("HTML Templating 🌐");
        });

        html::body(|| {
            html::h1("🌐 http1 template");
            html::p("This page was created with http1 templating engine");

            html::hr(());

            html::h2("http1 templates support:");
            html::ol(|| {
                html::li("All common html tags");
                html::li("Attributes like class, style or id");
                html::li(|| {
                    html::span(|| {
                        html::content("Custom attributes with ");
                        html::mark(|| {
                            html::code("html::attr(name, value)");
                        });
                    });
                });
                html::li(|| {
                    html::span(|| {
                        html::content("Custom tags with ");
                        html::mark(|| {
                            html::code("html::html_element(tag, content: T)");
                        });
                    });
                });
            });

            html::h3(|| {
                html::content("Other pages:");
                html::ul(|| {
                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/images");
                            html::content("📸 Images");
                        });
                    });

                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/styled");
                            html::content("💅 Styled");
                        });
                    });
                });
            });
        });
    })
}

fn images_page() -> impl IntoResponse {
    html::html(|| {
        html::head(|| {
            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::title("HTML Templating | Image 🌐");
        });

        html::body(|| {
            html::h1("📸 Image Gallery");
            html::p("This page showcases some images using http1 templating engine.");

            html::hr(());

            html::div(|| {
                html::img(|| {
                    html::attr("src", "https://via.placeholder.com/300");
                    html::attr("alt", "Placeholder Image 1");
                });

                html::img(|| {
                    html::attr("src", "https://via.placeholder.com/300");
                    html::attr("alt", "Placeholder Image 2");
                });

                html::img(|| {
                    html::attr("src", "https://via.placeholder.com/300");
                    html::attr("alt", "Placeholder Image 3");
                });
            });

            html::h3(|| {
                html::content("Other pages:");
                html::ul(|| {
                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/");
                            html::content("🏠 Home");
                        });
                    });

                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/styled");
                            html::content("💅 Styled");
                        });
                    });
                });
            });
        });
    })
}

fn styled_page() -> impl IntoResponse {
    html::html(|| {
        html::head(|| {
            html::meta(|| {
                html::attr("charset", "utf-8");
            });

            html::title("HTML Templating | Styled 🌐");
            html::style(
                r#"
                body {
                    font-family: Arial, sans-serif;
                    background: linear-gradient(90deg, #ff7e5f, #feb47b);
                    background-size: 200% 200%;
                    color: #333;
                    margin: 0;
                    padding: 0;
                }
                h1 {
                    color: white;
                    text-align: center;
                    margin-top: 20px;
                }
                p {
                    text-align: center;
                    color: white;
                    font-size: 1.2em;
                }
                ul {
                    display: flex;
                    justify-content: center;
                    list-style: none;
                    padding: 0;
                }
                li {
                    margin: 0 15px;
                }
                a {
                    color: white;
                    text-decoration: none;
                    font-weight: bold;
                }
            "#,
            );
        });

        html::body(|| {
            html::h1("💅 Styled Page");
            html::p("Welcome to the styled page, with custom CSS for a vibrant look!");

            html::hr(());

            html::h3(|| {
                html::content("Other pages:");
                html::ul(|| {
                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/");
                            html::content("🏠 Home");
                        });
                    });

                    html::li(|| {
                        html::a(|| {
                            html::attr("href", "/images");
                            html::content("📸 Images");
                        });
                    });
                });
            });
        });
    })
}
