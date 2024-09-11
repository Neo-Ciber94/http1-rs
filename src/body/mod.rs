pub mod http_body;

use std::{convert::Infallible, error::Error, fmt::Debug};

use crate::error::BoxError;
use http_body::HttpBody;

struct BoxBodyInner<B: HttpBody>(B);

impl<B: HttpBody> HttpBody for BoxBodyInner<B>
where
    B: HttpBody,
    B::Err: Error + Send + Sync + 'static,
    B::Data: Into<Vec<u8>>,
{
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.0
            .read()
            .map(|data| data.map(|x| x.into()))
            .map_err(|e| e.into())
    }
}

struct BoxBody(Box<dyn HttpBody<Err = BoxError, Data = Vec<u8>>>);

fn box_body<B>(body: B) -> BoxBody
where
    B: HttpBody + 'static,
    B::Err: Error + Send + Sync + 'static,
    B::Data: Into<Vec<u8>>,
{
    BoxBody(Box::new(BoxBodyInner(body)))
}

pub struct Body {
    inner: BoxBody,
}

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody + 'static,
        B::Err: Error + Send + Sync + 'static,
        B::Data: Into<Vec<u8>>,
    {
        let inner = box_body(body);
        Body { inner }
    }
}

impl HttpBody for Body {
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        self.inner.0.read()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

struct SizedBody(Option<Vec<u8>>);

impl HttpBody for SizedBody {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(self.0.take())
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        let bytes = value.bytes().collect::<_>();
        Body::new(SizedBody(Some(bytes)))
    }
}
