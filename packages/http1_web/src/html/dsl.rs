use super::element::{Attribute, Element, HTMLElement, Node};
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
};

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

impl<'a> IntoAttrValue for &'a String {
    fn into_attr_value(self) -> AttrValue {
        AttrValue::String(self.to_owned())
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

macro_rules! impl_into_attr_value_for_primitive {
    ($($T:ident),*) => {
       $(
            impl IntoAttrValue for $T {
                fn into_attr_value(self) -> AttrValue {
                    AttrValue::String(self.to_string())
                }
            }
       )*
    };
}

impl_into_attr_value_for_primitive!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64, char
);

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
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64, bool, char
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
struct Global {
    elements: Vec<Element>,
    context: Option<HashMap<TypeId, Box<dyn Any>>>,
}

thread_local! {
    static ROOT: RefCell<Global> = RefCell::new(Global {
        elements: Vec::new(),
        context: None
    });
}

fn __html_element<T: IntoChildren>(
    tag: impl Into<String>,
    is_void: bool,
    content: T,
) -> HTMLElement {
    ROOT.with_borrow_mut(move |global: &mut Global| {
        global
            .elements
            .push(Element::builder(tag).is_void(is_void).build());

        global.context.get_or_insert_with(Default::default);
    });

    let children = content.into_children();

    let result: Option<Element> = ROOT.with_borrow_mut(|global: &mut Global| {
        // Insert the current children in the last node
        if let Some(parent) = global.elements.last_mut() {
            match children {
                Children::Node(node) => parent.children_mut().push(node),
                Children::List(vec) => parent.children_mut().extend(vec),
                Children::None => {}
            }
        }

        // Then if there is any elements we insert the last one into the previous one
        if global.elements.len() > 1 {
            if let Some(el) = global.elements.pop() {
                if let Some(parent) = global.elements.last_mut() {
                    if !parent.is_void() {
                        parent.children_mut().push(el.into());
                    }
                }
            }

            return None;
        }

        // Return the last node
        global.elements.pop()
    });

    let element: HTMLElement = result.into();

    // Only the root element return a non empty, we can clear the context.
    if let HTMLElement::Element(_) = &element {
        ROOT.with_borrow_mut(|global: &mut Global| {
            global.context.take();
        });
    }

    element
}

/// Declare a `html` element.
pub fn html_element<T: IntoChildren>(tag: impl Into<String>, content: T) -> HTMLElement {
    __html_element(tag, false, content)
}

/// Declare a void `html` element.
pub fn html_void_element<T: IntoChildren>(tag: impl Into<String>, content: T) -> HTMLElement {
    __html_element(tag, true, content)
}

/// Declare a text node for the current html element.
pub fn content(text: impl Into<String>) {
    ROOT.with_borrow_mut(|global: &mut Global| {
        if let Some(parent) = global.elements.last_mut() {
            let text = text.into();
            parent.children_mut().push(text.into());
        }
    })
}

/// Sets an attribute in the current html element.
pub fn attr(name: impl Into<String>, value: impl IntoAttrValue) {
    ROOT.with_borrow_mut(|global: &mut Global| {
        if let Some(parent) = global.elements.last_mut() {
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
                    if !b {
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
            pub fn $tag<T: IntoChildren>(content: T) -> HTMLElement {
                html_element(stringify!($tag), content)
            }
       )*
    };
}

macro_rules! define_html_void_element_fn {
    ($($tag:ident),*) => {
       $(
            pub fn $tag<T: IntoChildren>(content: T) -> HTMLElement {
                html_void_element(stringify!($tag), content)
            }
       )*
    };
}

#[rustfmt::skip]
define_html_element_fn!(
    // Document Structure
    html, body, head, title, 
    
    // Metadata and Scripting
    style, script, noscript, template, 
    
    // Headings
    h1, h2, h3, h4, h5, h6, 
    
    // Sections and Grouping Content
    div, section, article, aside, header, footer, nav, main, 
    
    // Text Content
    p, span, a, i, u, b, strong, em, blockquote, pre, code, q, abbr, cite, dfn, mark, 
    
    // Lists
    ol, ul, li, dl, dt, dd, 
    
    // Table Content
    table, tr, td, th, thead, tbody, tfoot, caption, colgroup, 
    
    // Forms
    form, textarea, button, label, select, option, optgroup, fieldset, legend, output, 
    
    // Media and Embedded Content
    figure, figcaption, video, audio, canvas, svg, math, iframe, embed, object, param, picture
);

#[rustfmt::skip]
define_html_void_element_fn!(
    // Void elements (self-closing)
    hr, br, img, input, meta, link, area, base, col, command, keygen, source, wbr
);

/// Set a value in the global context.
pub fn set_context<T: 'static + Clone>(value: T) -> Option<T> {
    ROOT.with_borrow_mut(move |global: &mut Global| {
        let ctx = global
            .context
            .as_mut()
            .expect("context can only be used inside a html element");

        ctx.insert(std::any::TypeId::of::<T>(), Box::new(value))
            .and_then(|x| x.downcast().ok())
            .map(|x| *x)
    })
}

/// Gets a value from the global context.
pub fn get_context<T: 'static + Clone>() -> Option<T> {
    ROOT.with_borrow(move |global: &Global| {
        let ctx = global
            .context
            .as_ref()
            .expect("context can only be used inside a html element");

        ctx.get(&std::any::TypeId::of::<T>())
            .and_then(|x| x.downcast_ref::<T>())
            .cloned()
    })
}

#[cfg(test)]
mod tests {
    use super::{attr, content, get_context, html_element, html_void_element, set_context};

    #[test]
    fn should_build_1_level_html() {
        let html = html_element("html", || {}).into_element().unwrap();
        assert_eq!(html.tag(), "html");
    }

    #[test]
    fn should_build_2_level_html() {
        let html = html_element("html", || {
            html_element("body", || {});
        })
        .into_element()
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
        .into_element()
        .unwrap();

        assert_eq!(
            html.to_string(),
            r#"<html>
  <head>
    <title>This is a Title</title>
  </head>
  <body>
    <h1 class="text-red">Hello World!
      <hr />
    </h1>
  </body>
</html>
"#
        )
    }

    #[test]
    #[should_panic]
    fn should_panic_if_use_get_context_outside_html() {
        get_context::<usize>();
    }

    #[test]
    #[should_panic]
    fn should_panic_if_use_set_context_outside_html() {
        set_context::<usize>(234);
    }

    #[test]
    #[should_panic]
    fn should_panic_if_use_set_context_outside_html_2() {
        super::div(|| {
            set_context("hello");
        });

        set_context::<usize>(234);
    }

    #[test]
    fn should_set_and_get_context() {
        let html = super::div(|| {
            set_context(String::from("Hello World!"));
            super::p(get_context::<String>());
        })
        .into_element()
        .unwrap();

        assert_eq!(
            html.to_plain_string(),
            "<div>\n<p>Hello World!</p>\n</div>\n"
        );
    }
}
