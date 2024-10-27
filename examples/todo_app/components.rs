#![allow(non_snake_case)]
use http1_web::{
    html::{self, element::HTMLElement, IntoChildren},
    impl_serde_enum_str, impl_serde_struct,
};

use crate::routes::AuthenticatedUser;

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
    pub toast: Option<ToastProps>,
}

pub fn Layout(LayoutProps { auth, toast }: LayoutProps, content: impl IntoChildren) -> HTMLElement {
    html::div(|| {
        html::div(|| {
            html::class("w-full h-16");
        });

        if let Some(props) = toast {
            Toast(props);
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

        content.into_children();
    })
}

#[derive(Debug, Clone)]
pub enum ToastKind {
    Success,
    Error,
    Warn,
}

impl_serde_enum_str!(ToastKind => {
    Success,
    Error,
    Warn,
});

#[derive(Debug, Clone)]
pub struct ToastProps {
    message: String,
    kind: ToastKind,
}

impl_serde_struct!(ToastProps => {
    message: String,
    kind: ToastKind,
});

impl ToastProps {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        ToastProps {
            message: message.into(),
            kind,
        }
    }
}

fn Toast(ToastProps { message, kind }: ToastProps) {
    html::div(|| {
        html::class("animate-fade-out");
        html::div(|| {
            let style = match kind {
                ToastKind::Success => "text-white bg-green-500",
                ToastKind::Error => "text-white bg-red-500",
                ToastKind::Warn => "text-black bg-amber-500",
            };

            html::class(format!("fixed top-10 left-1/2 -translate-x-1/2 p-4 rounded-md shadow-md flex flex-row items-center justify-center cursor-pointer hover:opacity-80 {style}"));
            html::content(message)
        });
    });
}
