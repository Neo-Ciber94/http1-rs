pub mod body_reader;
pub mod body_writer;
pub mod buf_body_reader;
pub mod chunked_body;
pub mod http_body;

use std::{borrow::Cow, fmt::Debug};

use crate::error::BoxError;
use http_body::{Bytes, HttpBody};

struct BoxBodyInner<B: HttpBody>(B);

impl<B: HttpBody> HttpBody for BoxBodyInner<B>
where
    B: HttpBody,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.0
            .read_next()
            .map(|data| data.map(|x| x.into()))
            .map_err(|e| e.into())
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct BoxBody(Box<dyn HttpBody<Err = BoxError, Data = Vec<u8>> + Send + 'static>);

fn box_body<B>(body: B) -> BoxBody
where
    B: HttpBody + Send + 'static,
    B::Err: Into<BoxError>,
    B::Data: Into<Vec<u8>>,
{
    BoxBody(Box::new(BoxBodyInner(body)))
}

pub struct Body {
    inner: BoxBody,
}

impl Body {
    pub fn empty() -> Self {
        Self::new(())
    }

    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody + Send + 'static,
        B::Err: Into<BoxError>,
        B::Data: Into<Vec<u8>>,
    {
        let inner = box_body(body);
        Body { inner }
    }
}

impl HttpBody for Body {
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.inner.0.read_next()
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner.0.size_hint()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body::new(Bytes::new(value))
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(value: &'a [u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl<'a> From<Cow<'a, str>> for Body {
    fn from(value: Cow<'a, str>) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Body::empty()
    }
}
