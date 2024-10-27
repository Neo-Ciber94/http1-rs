use std::{borrow::Cow, collections::BTreeMap, fmt::Display};

use crate::IntoResponse;

use super::Html;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NameValueAttr {
    name: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BooleanAttr {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Attribute {
    NameValue(NameValueAttr),
    Boolean(BooleanAttr),
}

impl Attribute {
    pub fn with_value(name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();
        assert_html_name(&name, "html attribute");

        Attribute::NameValue(NameValueAttr { name, value })
    }

    pub fn boolean(name: impl Into<String>) -> Self {
        let name = name.into();
        assert_html_name(&name, "html attribute");
        Attribute::Boolean(BooleanAttr { name })
    }

    pub fn name(&self) -> &str {
        match self {
            Attribute::NameValue(name_value_attr) => name_value_attr.name.as_str(),
            Attribute::Boolean(bool_attr) => bool_attr.name.as_str(),
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Attribute::NameValue(name_value_attr) => Some(name_value_attr.value.as_str()),
            Attribute::Boolean(_) => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Attribute::Boolean(_))
    }
}

/// An html DOM node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    Element(Element),
    Text(String),
}

impl Node {
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    pub fn as_element(&self) -> Option<&Element> {
        match self {
            Node::Element(element) => Some(element),
            Node::Text(_) => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Node::Element(_) => None,
            Node::Text(text) => Some(text.as_str()),
        }
    }

    pub fn into_element(self) -> Option<Element> {
        match self {
            Node::Element(element) => Some(element),
            Node::Text(_) => None,
        }
    }

    pub fn into_text(self) -> Option<String> {
        match self {
            Node::Element(_) => None,
            Node::Text(text) => Some(text),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(element) => element.fmt(f),
            Node::Text(text) => write!(f, "{text}"),
        }
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Text(value)
    }
}

impl<'a> From<&'a str> for Node {
    fn from(value: &'a str) -> Self {
        Node::Text(value.into())
    }
}

impl<'a> From<Cow<'a, str>> for Node {
    fn from(value: Cow<'a, str>) -> Self {
        Node::Text(value.to_string())
    }
}

impl From<Element> for Node {
    fn from(value: Element) -> Self {
        Node::Element(value)
    }
}

impl From<Builder> for Node {
    fn from(value: Builder) -> Self {
        Node::Element(value.build())
    }
}

/// An DOM element, eg: `<html>`, `<body>`, `<div>`, etc.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Element {
    tag: String,
    attributes: BTreeMap<String, Attribute>,
    children: Vec<Node>,
    is_void: bool,
}

impl Element {
    pub fn new(tag: impl Into<String>) -> Self {
        Self::builder(tag).build()
    }

    pub fn builder(tag: impl Into<String>) -> Builder {
        Builder::new(tag)
    }

    pub fn tag(&self) -> &String {
        &self.tag
    }

    pub fn is_void(&self) -> bool {
        self.is_void
    }

    pub fn attributes(&self) -> &BTreeMap<String, Attribute> {
        &self.attributes
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn attributes_mut(&mut self) -> &mut BTreeMap<String, Attribute> {
        &mut self.attributes
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    pub fn to_plain_string(&self) -> String {
        let mut buf = String::new();
        write_element(self, &mut buf, "", 0).expect("failed to write string");
        buf
    }
}

impl IntoResponse for Element {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let html = self.to_string();
        Html(html).into_response()
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_element(self, f, "  ", 0)
    }
}

/// An html element or empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HTMLElement {
    Element(Element),
    None,
}

impl HTMLElement {
    pub fn into_element(self) -> Option<Element> {
        match self {
            HTMLElement::Element(element) => Some(element),
            HTMLElement::None => None,
        }
    }
}

impl From<Element> for HTMLElement {
    fn from(value: Element) -> Self {
        HTMLElement::Element(value)
    }
}

impl From<Option<Element>> for HTMLElement {
    fn from(value: Option<Element>) -> Self {
        match value {
            Some(e) => HTMLElement::Element(e),
            None => HTMLElement::None,
        }
    }
}

impl IntoResponse for HTMLElement {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        let html = match self {
            HTMLElement::Element(element) => element.to_string(),
            HTMLElement::None => {
                log::warn!("DOM element was empty");
                String::new()
            }
        };

        Html(html).into_response()
    }
}

fn write_element<W: std::fmt::Write>(
    el: &Element,
    f: &mut W,
    indent_str: &'static str,
    indentation_level: usize,
) -> std::fmt::Result {
    fn write_indent<W: std::fmt::Write>(
        f: &mut W,
        indent_str: &'static str,
        indentation_level: usize,
    ) -> std::fmt::Result {
        for _ in 0..indentation_level {
            f.write_str(indent_str)?;
        }

        Ok(())
    }

    write_indent(f, indent_str, indentation_level)?;

    // Opening
    write!(f, "<{}", el.tag)?;

    for attr in el.attributes.values() {
        match attr {
            Attribute::NameValue(NameValueAttr { name, value }) => {
                let name = escape_html(name);
                let value = escape_html(value);
                write!(f, " {name}=\"{value}\"")?;
            }
            Attribute::Boolean(BooleanAttr { name }) => {
                let name = escape_html(name);
                write!(f, " {name}")?;
            }
        }
    }

    if !el.is_void {
        write!(f, ">")?;
        let len = el.children.len();

        if len > 0 {
            writeln!(f, "")?;
        }

        for node in el.children() {
            match node {
                Node::Element(element) => {
                    write_element(element, f, indent_str, indentation_level + 1)?
                }
                Node::Text(text) => {
                    write_indent(f, indent_str, indentation_level + 1)?;
                    let text = escape_html(text);
                    writeln!(f, "{text}")?;
                }
            }
        }

        if len > 0 {
            write_indent(f, indent_str, indentation_level)?;
        }

        writeln!(f, "</{}>", el.tag)?;
    } else {
        writeln!(f, " />")?;
    }

    Ok(())
}

fn escape_html<'a>(input: &'a str) -> Cow<'a, str> {
    if !input.contains(|c| matches!(c, '&' | '<' | '>' | '"' | '\'')) {
        return Cow::Borrowed(input);
    }

    let mut result = String::new();

    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }

    Cow::Owned(result)
}

pub struct Builder {
    tag: String,
    attributes: BTreeMap<String, Attribute>,
    children: Vec<Node>,
    is_void: bool,
}

impl Builder {
    pub fn new(tag: impl Into<String>) -> Self {
        let tag = tag.into();
        assert_html_name(&tag, "html tag");

        Builder {
            tag,
            attributes: Default::default(),
            children: Default::default(),
            is_void: false,
        }
    }

    pub fn is_void(mut self, is_void: bool) -> Self {
        self.is_void = is_void;
        self
    }

    pub fn attribute(mut self, attr: Attribute) -> Self {
        self.attributes.insert(attr.name().to_owned(), attr);
        self
    }

    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn build(self) -> Element {
        Element {
            tag: self.tag,
            attributes: self.attributes,
            children: self.children,
            is_void: self.is_void,
        }
    }
}

fn assert_html_name(name: &str, debug_name: &'static str) {
    assert!(!name.is_empty(), "{debug_name} cannot be empty");

    let first = name.as_bytes()[0];

    assert!(
        first.is_ascii_alphabetic(),
        "{debug_name} should start with a letter"
    );

    for b in name.as_bytes() {
        assert!(
            b.is_ascii(),
            "{debug_name} contains a non-ascii character: `{}`",
            *b as char
        );
        assert!(
            !b.is_ascii_whitespace(),
            "{debug_name} cannot contains whitespace"
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::html::element::Attribute;

    use super::Element;

    #[test]
    fn should_display_element() {
        let el = Element::builder("html")
            .child(Element::builder("head").child(Element::builder("title").child("This is HTML")))
            .child(
                Element::builder("body").child(
                    Element::builder("h1")
                        .attribute(Attribute::with_value("class", "text-red"))
                        .child("Hello World!")
                        .child(Element::builder("hr").is_void(true)),
                ),
            )
            .build();

        assert_eq!(
            el.to_string(),
            r#"<html>
  <head>
    <title>
      This is HTML
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
