use std::{
    fmt::Debug,
    sync::mpsc::{channel, Receiver, Sender},
};

pub enum Body {
    Payload(Vec<u8>),
    Stream(Receiver<Vec<u8>>),
}

impl Body {
    pub fn new(payload: impl Into<Vec<u8>>) -> Self {
        Body::Payload(payload.into())
    }

    pub fn stream() -> (Sender<Vec<u8>>, Self) {
        let (sender, recv) = channel::<Vec<u8>>();
        let body = Body::Stream(recv);
        (sender, body)
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut writer = f.debug_struct("Body");
        match self {
            Body::Payload(data) => {
                writer.field("payload", data);
            }
            Body::Stream(_) => {
                writer.field("stream",&"..");
            }
        }
        writer.finish()
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        Body::new(value)
    }
}

impl<'a> From<&'a String> for Body {
    fn from(value: &'a String) -> Self {
        Body::new(value.as_str())
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Body::new(value)
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body::new(value)
    }
}
