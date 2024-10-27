#![allow(non_snake_case)]
use http1_web::{
    cookies::Cookies,
    from_request::FromRequestRef,
    html::{self, element::HTMLElement, IntoChildren},
    ErrorStatusCode,
};

use crate::{consts::COOKIE_FLASH_MESSAGE, routes::models::AuthenticatedUser};

pub fn Head(content: impl IntoChildren) -> HTMLElement {
    html::head(|| {
        content.into_children();

        html::meta(|| html::attr("charset", "UTF-8"));
        html::link(|| {
            html::attr("rel", "icon");
            html::attr("href", "/favicon.png");
        });

        html::link(|| {
            html::attr("href", "/styles.css");
            html::attr("rel", "stylesheet");
        });

        html::script(|| {
            html::attr("src", "https://cdn.tailwindcss.com");
        });
    })
}

pub fn Title(title: impl Into<String>) -> HTMLElement {
    html::title(title.into())
}

pub struct LayoutProps {
    pub auth: Option<AuthenticatedUser>,
    pub alert: Option<AlertProps>,
    pub class: Option<String>,
}

pub fn Layout(
    LayoutProps { auth, alert, class }: LayoutProps,
    content: impl IntoChildren,
) -> HTMLElement {
    html::div(|| {
        html::class("min-h-screen flex flex-col");

        html::div(|| {
            html::class("w-full h-16");
        });

        if let Some(props) = alert {
            Alert(props);
        }

        html::header(|| {
            html::class("w-full h-16 shadow fixed top-0 left-0 flex flex-row p-2 justify-between items-center");

            html::a(|| {
                html::attr("href", "/");
                html::h4(|| {
                    html::content("TodoApp");
                    html::class("text-xl font-bold text-blue-500");
                });
            });

            if let Some(AuthenticatedUser(user)) = auth {
                html::div(|| {
                    html::class("flex flex-row gap-2 items-center");
                    html::h4(|| {
                        html::content(user.username);
                        html::class("font-bold text-xl");
                    });
                    html::a(|| {
                        html::attr("href", "/api/logout");

                        html::button(|| {
                            html::content("Logout");
                            html::class("p-3 bg-black text-white rounded hover:bg-slate-800 transition inline-block shadow-md");
                        });
                    });
                });
            } else {
                html::div("");
            }
        });

        html::div(|| {
            html::class(format!(
                "flex flex-col items-center justify-center p-4 h-full w-full {}",
                class.unwrap_or_default()
            ));
            content.into_children();
        });
    })
}

#[derive(Debug, Clone)]
pub enum AlertKind {
    Success,
    Error,
    Warn,
}

serde::impl_serde_enum_str!(AlertKind => {
    Success,
    Error,
    Warn,
});

#[derive(Debug, Clone)]
pub struct AlertProps {
    message: String,
    kind: AlertKind,
}

serde::impl_serde_struct!(AlertProps => {
    message: String,
    kind: AlertKind,
});

impl AlertProps {
    pub fn new(message: impl Into<String>, kind: AlertKind) -> Self {
        AlertProps {
            message: message.into(),
            kind,
        }
    }
}

fn Alert(AlertProps { message, kind }: AlertProps) {
    html::div(|| {
        html::class("animate-fade-out");
        html::div(|| {
            let style = match kind {
                AlertKind::Success => "text-white bg-green-500",
                AlertKind::Error => "text-white bg-red-500",
                AlertKind::Warn => "text-black bg-amber-500",
            };

            html::class(format!("fixed top-10 left-1/2 -translate-x-1/2 p-4 rounded-md shadow-md flex flex-row items-center justify-center cursor-pointer hover:opacity-80 {style}"));
            html::content(message)
        });
    });
}

impl FromRequestRef for AlertProps {
    type Rejection = ErrorStatusCode;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_ref(req).unwrap_or_default();
        match cookies.get(COOKIE_FLASH_MESSAGE) {
            Some(cookie) => match serde::json::from_str::<AlertProps>(cookie.value()) {
                Ok(alert) => Ok(alert),
                Err(err) => {
                    log::error!("AlertProps should be injected with Option<AlertProps>: {err}");
                    Err(ErrorStatusCode::InternalServerError)
                }
            },
            None => Err(ErrorStatusCode::InternalServerError),
        }
    }
}
