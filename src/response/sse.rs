pub struct Sse {}

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

    pub fn with_data(data: impl Into<String>) -> Result<Self, InvalidSSeEvent> {
        Builder::new().data(data)
    }

    pub fn with_event_data(
        event: impl Into<String>,
        data: impl Into<String>,
    ) -> Result<Self, InvalidSSeEvent> {
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

#[derive(Debug, Default)]
struct Parts {
    id: Option<String>,
    event: Option<String>,
    retry: Option<usize>,
}

#[derive(Debug)]
pub enum InvalidSSeEvent {
    InvalidId(&'static str),
    InvalidEvent(&'static str),
    InvalidData(&'static str),
}

#[derive(Debug)]
pub struct Builder(Result<Parts, InvalidSSeEvent>);

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    fn update<F: FnOnce(&mut Parts) -> Result<(), InvalidSSeEvent>>(mut self, f: F) -> Self {
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
                return Err(InvalidSSeEvent::InvalidId(
                    "'id' cannot contains a line-break",
                ));
            }

            parts.id = Some(id);
            Ok(())
        })
    }

    pub fn event(self, event: impl Into<String>) -> Self {
        let event: String = event.into();

        self.update(|parts| {
            if has_line_break(&event) {
                return Err(InvalidSSeEvent::InvalidEvent(
                    "'event' cannot contains a line-break",
                ));
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

    pub fn data(self, data: impl Into<String>) -> Result<SseEvent, InvalidSSeEvent> {
        let Parts { event, id, retry } = self.0?;

        let data = data.into();

        if has_line_break(&data) {
            return Err(InvalidSSeEvent::InvalidEvent(
                "'data' cannot contains a line-break",
            ));
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
