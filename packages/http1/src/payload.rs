use crate::{
    body::{http_body::HttpBody, Body},
    error::BoxError,
};

/// Represents a request body.
#[derive(Default, Debug)]
pub enum Payload {
    Data(Body),

    #[default]
    None,
}

impl Payload {
    /// Whether this payload has data.
    pub fn is_empty(&self) -> bool {
        matches!(self, Payload::None)
    }

    /// Takes the body if any.
    pub fn take(&mut self) -> Option<Body> {
        let this = std::mem::take(self);
        match this {
            Payload::Data(body) => Some(body),
            Payload::None => None,
        }
    }

    /// Returns the body or panic if the payload have no body.
    pub fn unwrap(self) -> Body {
        match self {
            Payload::Data(body) => body,
            Payload::None => panic!("payload have no body"),
        }
    }
}

impl HttpBody for Payload {
    type Err = BoxError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<<Payload as HttpBody>::Data>, Self::Err> {
        match self {
            Payload::Data(body) => body.read_next(),
            Payload::None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self {
            Payload::Data(body) => body.size_hint(),
            Payload::None => None,
        }
    }
}
