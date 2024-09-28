use std::{
    fmt::Display,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use crate::body::{http_body::HttpBody, Body};

#[derive(Debug)]
pub struct SseSendError;

pub struct SseStream(Option<Receiver<SseEvent>>);

impl SseStream {
    pub fn new() -> (SseBroadcast, Self) {
        let (sender, receiver) = channel();

        let sse_broadcast = SseBroadcast(sender);
        (sse_broadcast, SseStream(Some(receiver)))
    }
}

impl From<SseStream> for Body {
    fn from(value: SseStream) -> Self {
        Body::new(value)
    }
}

#[derive(Debug)]
pub struct InvalidSseStreamError;

impl Display for InvalidSseStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send server-sent event")
    }
}

impl std::error::Error for InvalidSseStreamError {}

impl HttpBody for SseStream {
    type Err = InvalidSseStreamError;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        fn to_bytes(event: SseEvent) -> Vec<u8> {
            event.to_string().into_bytes()
        }

        match self.0.as_mut() {
            Some(receiver) => {
                match receiver.try_recv() {
                    Ok(event) => Ok(Some(to_bytes(event))),
                    Err(TryRecvError::Disconnected) => {
                        let _ = self.0.take(); // drop the receiver
                        Err(InvalidSseStreamError)
                    }
                    Err(TryRecvError::Empty) => {
                        // Wait until the next message
                        match receiver.recv() {
                            Ok(event) => Ok(Some(to_bytes(event))),
                            Err(_) => {
                                let _ = self.0.take(); // drop the receiver
                                Err(InvalidSseStreamError)
                            }
                        }
                    }
                }
            }
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct SseBroadcast(Sender<SseEvent>);

impl SseBroadcast {
    pub fn send(&self, event: SseEvent) -> Result<(), SseSendError> {
        self.0.send(event).map_err(|_| SseSendError)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SseEvent {
    id: Option<String>,
    event: Option<String>,
    data: String,
    retry: Option<usize>,
}

impl SseEvent {
    pub fn new() -> Builder {
        Builder::new()
    }

    pub fn with_data(data: impl Into<String>) -> Result<Self, InvalidSseEvent> {
        Builder::new().data(data)
    }

    pub fn with_event_data(
        event: impl Into<String>,
        data: impl Into<String>,
    ) -> Result<Self, InvalidSseEvent> {
        Builder::new().event(event).data(data)
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    pub fn data(&self) -> &str {
        self.data.as_str()
    }

    pub fn retry(&self) -> Option<usize> {
        self.retry
    }
}

impl Display for SseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "id: {id}\r\n")?;
        }

        if let Some(event) = &self.event {
            write!(f, "event: {event}\r\n")?;
        }

        if let Some(retry) = &self.retry {
            write!(f, "retry: {retry}\r\n")?;
        }

        write!(f, "data: {}\r\n", self.data)?;

        Ok(())
    }
}

#[derive(Debug, Default)]
struct Parts {
    id: Option<String>,
    event: Option<String>,
    retry: Option<usize>,
}

#[derive(Debug)]
pub enum InvalidSseEvent {
    InvalidIdLineBreak,
    InvalidEventLineBreak,
    InvalidDataLineBreak,
}

impl Display for InvalidSseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidSseEvent::InvalidIdLineBreak => write!(f, "'id' cannot contain a line-break"),
            InvalidSseEvent::InvalidEventLineBreak => {
                write!(f, "'event' cannot contain a line-break")
            }
            InvalidSseEvent::InvalidDataLineBreak => {
                write!(f, "'data' cannot contain a line-break")
            }
        }
    }
}

#[derive(Debug)]
pub struct Builder(Result<Parts, InvalidSseEvent>);

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    fn update<F: FnOnce(&mut Parts) -> Result<(), InvalidSseEvent>>(mut self, f: F) -> Self {
        self.0 = self.0.and_then(|mut parts| {
            f(&mut parts)?;
            Ok(parts)
        });

        self
    }

    pub fn id(self, id: impl Into<String>) -> Self {
        let id: String = id.into();

        self.update(|parts| {
            if has_line_break(&id) {
                return Err(InvalidSseEvent::InvalidIdLineBreak);
            }

            parts.id = Some(id);
            Ok(())
        })
    }

    pub fn event(self, event: impl Into<String>) -> Self {
        let event: String = event.into();

        self.update(|parts| {
            if has_line_break(&event) {
                return Err(InvalidSseEvent::InvalidEventLineBreak);
            }

            parts.event = Some(event);
            Ok(())
        })
    }

    pub fn retry(self, retry: usize) -> Self {
        self.update(|parts| {
            parts.retry = Some(retry);
            Ok(())
        })
    }

    pub fn data(self, data: impl Into<String>) -> Result<SseEvent, InvalidSseEvent> {
        let Parts { event, id, retry } = self.0?;

        let data = data.into();

        if has_line_break(&data) {
            return Err(InvalidSseEvent::InvalidDataLineBreak);
        }

        Ok(SseEvent {
            id,
            event,
            data,
            retry,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self(Ok(Parts::default()))
    }
}

fn has_line_break(s: &str) -> bool {
    s.bytes().find(|c| *c == b'\n').is_some()
}
