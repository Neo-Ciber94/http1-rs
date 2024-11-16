use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

impl Message {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data.as_slice(),
            Message::Text(s) => s.as_bytes(),
            Message::Close => &[],
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(s) => Some(s.as_str()),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                std::str::from_utf8(data).ok()
            }
            Message::Close => Some(""),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Message::Text(text) => text.into_bytes(),
            Message::Binary(vec) | Message::Ping(vec) | Message::Pong(vec) => vec,
            Message::Close => Vec::new(),
        }
    }

    pub fn into_text(self) -> Option<String> {
        match self {
            Message::Text(text) => Some(text),
            Message::Binary(vec) | Message::Ping(vec) | Message::Pong(vec) => {
                String::from_utf8(vec).ok()
            }
            Message::Close => Some(String::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    pub fn is_ping(&self) -> bool {
        matches!(self, Message::Ping(_))
    }

    pub fn is_pong(&self) -> bool {
        matches!(self, Message::Pong(_))
    }

    pub fn is_close(&self) -> bool {
        matches!(self, Message::Close)
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Message::Text(value)
    }
}

impl<'a> From<&'a str> for Message {
    fn from(value: &'a str) -> Self {
        value.to_owned().into()
    }
}

impl From<Vec<u8>> for Message {
    fn from(value: Vec<u8>) -> Self {
        Message::Binary(value)
    }
}

impl<'a> From<&'a [u8]> for Message {
    fn from(value: &'a [u8]) -> Self {
        value.to_vec().into()
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(text) = self.as_text() {
            write!(f, "{text}")
        } else {
            write!(f, "Binary<length={}>", self.len())
        }
    }
}
