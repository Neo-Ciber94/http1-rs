use super::element::{Attribute, Element, Node};
use std::{borrow::Cow, cell::RefCell};

pub enum AttrValue {
    String(String),
    Bool(bool),
}

pub trait IntoAttrValue {
    fn into_attr_value(self) -> AttrValue;
}

impl IntoAttrValue for String {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self)
    }
}

impl<'a> IntoAttrValue for &'a str {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self.to_owned())
    }
}

impl<'a> IntoAttrValue for Cow<'a, str> {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self.to_string())
    }
}

impl IntoAttrValue for bool {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::Bool(self)
    }
}

pub enum Children {
    None,
    Node(Node),
    List(Vec<Node>),
}

pub trait IntoChildren {
    fn into_children(self) -> Children;
}

impl IntoChildren for String {
    fn into_children(self) -> Children {
        Children::Node(Node::Text(self))
    }
}

impl<'a> IntoChildren for &'a str {
    fn into_children(self) -> Children {
        Children::Node(Node::Text(self.to_owned()))
    }
}

impl<'a> IntoChildren for Cow<'a, str> {
    fn into_children(self) -> Children {
        Children::Node(Node::Text(self.to_string()))
    }
}

macro_rules! impl_into_children_to_display {
    ($($T:ident),*) => {
        $(
            impl IntoChildren for $T {
                fn into_children(self) -> Children {
                    Children::Node(Node::Text(self.to_string()))
                }
            }
        )*
    };
}

impl_into_children_to_display!(
    u8, u16, u32, u64, u128, 
    i8, i16, i32, i64, i128, 
    usize, isize,
    f32, f64, 
    bool, char
);

impl IntoChildren for () {
    fn into_children(self) -> Children {
        Children::None
    }
}

impl IntoChildren for Node {
    fn into_children(self) -> Children {
        Children::Node(self)
    }
}

impl IntoChildren for Element {
    fn into_children(self) -> Children {
        Children::Node(Node::Element(self))
    }
}

impl<T: IntoChildren> IntoChildren for Option<T> {
    fn into_children(self) -> Children {
        match self {
            Some(x) => x.into_children(),
            None => Children::None,
        }
    }
}

impl<T: IntoChildren, const N: usize> IntoChildren for [T; N] {
    fn into_children(self) -> Children {
        let mut nodes = vec![];

        for x in self {
            match x.into_children() {
                Children::None => {}
                Children::Node(node) => nodes.push(node),
                Children::List(vec) => nodes.extend(vec),
            }
        }

        if nodes.is_empty() {
            Children::None
        } else {
            Children::List(nodes)
        }
    }
}

impl<T: IntoChildren> IntoChildren for Vec<T> {
    fn into_children(self) -> Children {
        let mut nodes = vec![];

        for x in self {
            match x.into_children() {
                Children::None => {}
                Children::Node(node) => nodes.push(node),
                Children::List(vec) => nodes.extend(vec),
            }
        }

        if nodes.is_empty() {
            Children::None
        } else {
            Children::List(nodes)
        }
    }
}

impl<F: FnOnce()> IntoChildren for F {
    fn into_children(self) -> Children {
        (self)();
        Children::None
    }
}

macro_rules! impl_into_children_tuple {
    ($($T:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($T),*> IntoChildren for ($($T),*,) where $($T: IntoChildren),* {
            fn into_children(self) -> Children {
                let mut nodes = vec![];

                let ($($T),*,) = self;

                $(
                    match IntoChildren::into_children($T) {
                        Children::None => {},
                        Children::Node(node) => nodes.push(node),
                        Children::List(vec) => nodes.extend(vec),
                    };
                )*

                if nodes.is_empty() {
                    Children::None
                } else {
                    Children::List(nodes)
                }
            }
        }
    };
}

impl_into_children_tuple!(T1);
impl_into_children_tuple!(T1, T2);
impl_into_children_tuple!(T1, T2, T3);
impl_into_children_tuple!(T1, T2, T3, T4);
impl_into_children_tuple!(T1, T2, T3, T4, T5);
impl_into_children_tuple!(T1, T2, T3, T4, T5, T6);
impl_into_children_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_into_children_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_into_children_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_into_children_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

#[derive(Debug)]
struct Context {
    elements: Vec<Element>,
}

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context { elements: Vec::new() });
}

fn __html_element<T: IntoChildren>(
    tag: impl Into<String>,
    is_void: bool,
    content: T,
) -> Option<Element> {
    CONTEXT.with_borrow_mut(move |ctx: &mut Context| {
        ctx.elements
            .push(Element::builder(tag).is_void(is_void).build());
    });

    let children = content.into_children();

    CONTEXT.with_borrow_mut(|ctx: &mut Context| {
        // Insert the current children in the last node
        if let Some(parent) = ctx.elements.last_mut() {
            match children {
                Children::Node(node) => parent.children_mut().push(node),
                Children::List(vec) => parent.children_mut().extend(vec),
                Children::None => {}
            }
        }

        // Then if there is any elements we insert the last one into the previous one
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

        // Return the last node
        ctx.elements.pop()
    })
}

/// Declare a `html` element.
pub fn html_element<T: IntoChildren>(tag: impl Into<String>, content: T) -> Option<Element> {
    __html_element(tag, false, content)
}

/// Declare a void `html` element.
pub fn html_void_element<T: IntoChildren>(tag: impl Into<String>, content: T) -> Option<Element> {
    __html_element(tag, true, content)
}

/// Declare a text node for the current html element.
pub fn content(text: impl Into<String>) {
    CONTEXT.with_borrow_mut(|ctx: &mut Context| {
        if let Some(parent) = ctx.elements.last_mut() {
            let text = text.into();
            parent.children_mut().push(text.into());
        }
    })
}

/// Sets an attribute in the current html element.
pub fn attr(name: impl Into<String>, value: impl IntoAttrValue) {
    CONTEXT.with_borrow_mut(|ctx: &mut Context| {
        if let Some(parent) = ctx.elements.last_mut() {
            let name = name.into();
            let attr_value = value.into_attr_value();

            match attr_value {
                AttrValue::String(s) => {
                    parent
                        .attributes_mut()
                        .insert(name.clone(), Attribute::with_value(name, s));
                }
                AttrValue::Bool(b) => {
                    // If the boolean is false we can omit the value
                    if b == false {
                        return;
                    }

                    parent
                        .attributes_mut()
                        .insert(name.clone(), Attribute::boolean(name));
                }
            }
        }
    })
}

/// Sets a `class` attribute in the current html element.
pub fn class(value: impl IntoAttrValue) {
    attr("class", value)
}

/// Sets a `id` attribute in the current html element.
pub fn id(value: impl IntoAttrValue) {
    attr("id", value)
}

/// Sets a `style` attribute in the current html element.
pub fn styles(value: impl IntoAttrValue) {
    attr("style", value)
}

macro_rules! define_html_element_fn {
    ($($tag:ident),*) => {
       $(
            pub fn $tag<T: IntoChildren>(content: T) -> Option<Element> {
                html_element(stringify!($tag), content)
            }
       )*
    };
}

macro_rules! define_html_void_element_fn {
    ($($tag:ident),*) => {
       $(
            pub fn $tag<T: IntoChildren>(content: T) -> Option<Element> {
                html_void_element(stringify!($tag), content)
            }
       )*
    };
}

#[doc = concat!("Declares a `<")]
pub fn hello<T: IntoChildren>(content: T) -> Option<Element> {
    html_element(stringify!(hello), content)
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
