use std::cell::RefCell;

use element::{Attribute, Element};
use http1::{headers::CONTENT_TYPE, response::Response};

use crate::into_response::IntoResponse;

pub mod element;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Html(String);

impl Html {
    pub fn raw(s: impl Into<String>) -> Self {
        Html(s.into())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl IntoResponse for Html {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        Response::builder()
            .insert_header(CONTENT_TYPE, "text/html")
            .body(self.0.into())
    }
}

#[derive(Debug)]
struct Context {
    elements: Vec<Element>,
}

thread_local! {
    static SCOPES: RefCell<Context> = RefCell::new(Context { elements: Vec::new() });
}

fn __html_element<F: FnOnce()>(tag: impl Into<String>, is_void: bool, block: F) -> Option<Element> {
    SCOPES.with_borrow_mut(move |ctx: &mut Context| {
        ctx.elements
            .push(Element::builder(tag).is_void(is_void).build());
    });

    block();

    SCOPES.with_borrow_mut(|ctx: &mut Context| {
        // If there is any elements we insert the last one into the previous one children
        if ctx.elements.len() > 1 {
            if let Some(el) = ctx.elements.pop() {
                if let Some(parent) = ctx.elements.last_mut() {
                    if !parent.is_void() {
                        parent.children_mut().push(el.into());
                    }
                }
            }

            return None;
        }

        ctx.elements.pop()
    })
}

/// Declare a `html` element.
pub fn html_element<F: FnOnce()>(tag: impl Into<String>, block: F) -> Option<Element> {
    __html_element(tag, false, block)
}

/// Declare a void `html` element.
pub fn html_void_element<F: FnOnce()>(tag: impl Into<String>, block: F) -> Option<Element> {
    __html_element(tag, true, block)
}

/// Declare a text node for the current html element.
pub fn content(text: impl Into<String>) {
    SCOPES.with_borrow_mut(|ctx: &mut Context| {
        if let Some(parent) = ctx.elements.last_mut() {
            let text = text.into();
            parent.children_mut().push(text.into());
        }
    })
}

/// Sets an attribute in the current html element.
pub fn attr(name: impl Into<String>, value: impl Into<String>) {
    SCOPES.with_borrow_mut(|ctx: &mut Context| {
        if let Some(parent) = ctx.elements.last_mut() {
            let name = name.into();
            parent
                .attributes_mut()
                .insert(name.clone(), Attribute::with_value(name, value));
        }
    })
}

macro_rules! define_html_element_fn {
    ($($tag:ident),*) => {
       $(
            pub fn $tag<F: FnOnce()>(block: F) -> Option<Element> {
                html_element(stringify!($tag), block)
            }
       )*
    };
}

macro_rules! define_html_void_element_fn {
    ($($tag:ident),*) => {
       $(
            pub fn $tag<F: FnOnce()>(block: F) -> Option<Element> {
                html_void_element(stringify!($tag), block)
            }
       )*
    };
}

#[rustfmt::skip]
define_html_element_fn!(
    // Document Structure
    html, body, head, title, 
    
    // Metadata and Scripting
    style, script,
    
     // Headings
    h1, h2, h3, h4, h5, h6, 
    
    // Sections and Grouping Content
    div, section, article, aside, header, footer, nav, 
    
    // Text Content
    p, span, a, i, u, b, strong, em, blockquote, pre, code, 
    
    // Lists
    ol, ul, li, dl, dt, dd, 
    
    // Table Content
    table, tr, td, th, thead, tbody, tfoot, caption, 
    
    // Forms
    form, textarea, button, label, select, option, 
    
    // Media and Embedded Content
    figure, figcaption
);

define_html_void_element_fn!(
    // Void elements (self-closing)
    hr, br, img, input, meta, link
);

#[cfg(test)]
mod tests {

    use super::{attr, content, html_element, html_void_element};

    #[test]
    fn should_build_1_level_html() {
        let html = html_element("html", || {}).unwrap();
        assert_eq!(html.tag(), "html");
    }

    #[test]
    fn should_build_2_level_html() {
        let html = html_element("html", || {
            html_element("body", || {});
        })
        .unwrap();

        assert_eq!(html.tag(), "html");
        assert_eq!(html.children().len(), 1);
        assert_eq!(
            html.children().as_slice()[0].as_element().unwrap().tag(),
            "body"
        );
    }

    #[test]
    fn should_build_element() {
        let html = html_element("html", || {
            html_element("head", || {
                html_element("title", || {
                    content("This is a Title");
                });
            });

            html_element("body", || {
                html_element("h1", || {
                    attr("class", "text-red");
                    content("Hello World!");
                    html_void_element("hr", || {});
                });
            });
        })
        .expect("no html nodes");

        assert_eq!(
            html.to_string(),
            r#"<html>
  <head>
    <title>
      This is a Title
    </title>
  </head>
  <body>
    <h1 class="text-red">
      Hello World!
      <hr />
    </h1>
  </body>
</html>
"#
        )
    }
}
