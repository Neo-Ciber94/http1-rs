use std::{
    fmt::{Debug, Display},
    io::{Read, Write},
    time::{Duration, Instant},
};

use http1::{common::uuid::Uuid, error::BoxError, protocol::upgrade::Upgrade};

use super::{
    frame::{CloseFrame, CloseFrameError, CloseStatusCode, Frame, OpCode},
    Message,
};

const BUFFER_SIZE: usize = 4 * 1024;
const MAX_PAYLOAD_LENGTH: usize = 64 * 1024; // 64 kb

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketConfig {
    pub buffer_size: usize,
    pub max_payload_length: Option<usize>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            buffer_size: BUFFER_SIZE,
            max_payload_length: Some(MAX_PAYLOAD_LENGTH),
        }
    }
}

#[derive(Debug)]
pub enum WebSocketError {
    PayloadTooBig { min: usize, actual: usize },
    InvalidPayloadLen(u64),
    InvalidOpCode(u8),
    UnmaskedClientPayload,
    CloseError(CloseFrameError),
    Timeout,
    Closed,
    IO(std::io::Error),
    Other(BoxError),
}

impl std::error::Error for WebSocketError {}

impl Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::PayloadTooBig { min, actual } => {
                write!(f, "websocket payload is too big: {actual} > {min}")
            }
            WebSocketError::IO(error) => write!(f, "websocket io error: {error}"),
            WebSocketError::InvalidPayloadLen(len) => write!(f, "Invalid payload len: {len}"),
            WebSocketError::InvalidOpCode(code) => write!(f, "invalid op_code: {code:X}"),
            WebSocketError::UnmaskedClientPayload => {
                write!(f, "client payload should always be encoded")
            }
            WebSocketError::CloseError(error) => write!(f, "failed to close websocket: `{error}`"),
            WebSocketError::Closed => write!(f, "websocket connection is closed"),
            WebSocketError::Other(error) => write!(f, "{error}"),
            WebSocketError::Timeout => write!(f, "websocket timeout"),
        }
    }
}

impl From<std::io::Error> for WebSocketError {
    fn from(value: std::io::Error) -> Self {
        WebSocketError::IO(value)
    }
}

#[derive(Debug)]
pub struct WebSocketSplitError;

pub struct WebSocket {
    upgrade: Upgrade,
    max_payload_length: Option<usize>,
    buf: Box<[u8]>,
}

impl WebSocket {
    /// Constructs a websocket from the given connection.
    pub fn new(upgrade: Upgrade) -> Self {
        Self::with_config(upgrade, Default::default())
    }

    /// Constructs a websocket with the given config.
    pub fn with_config(upgrade: Upgrade, config: WebSocketConfig) -> Self {
        let WebSocketConfig {
            buffer_size,
            max_payload_length,
        } = config;

        assert!(buffer_size > 0, "websocket buffer size must be non-zero");

        let buf = vec![0; buffer_size].into_boxed_slice();

        WebSocket {
            upgrade,
            buf,
            max_payload_length,
        }
    }

    /// Reads a message.
    #[track_caller]
    pub fn recv(&mut self) -> Result<Message, WebSocketError> {
        self.read(None)
    }

    #[track_caller]
    /// Reads a message and errors if timeout.
    pub fn recv_timeout(&mut self, timeout: Duration) -> Result<Message, WebSocketError> {
        self.read(Some(timeout))
    }

    #[track_caller]
    fn read(&mut self, timeout: Option<Duration>) -> Result<Message, WebSocketError> {
        let mut msg_data = Vec::new();
        let mut msg_op_code: Option<OpCode> = None;
        let now = Instant::now();

        loop {
            if let Some(timeout) = timeout {
                if now.elapsed() > timeout {
                    return Err(WebSocketError::Timeout);
                }
            }

            let frame = self.read_frame()?;

            let Frame {
                fin,
                op_code,
                payload,
                ..
            } = frame;

            msg_data.extend(payload);
            msg_op_code.get_or_insert(op_code);

            if fin {
                break;
            }
        }

        let op_code = msg_op_code.expect("message op_code was empty");
        match op_code {
            OpCode::Binary => Ok(Message::Binary(msg_data)),
            OpCode::Text => {
                let text =
                    String::from_utf8(msg_data).map_err(|err| WebSocketError::Other(err.into()))?;
                Ok(Message::Text(text))
            }
            OpCode::Ping => Ok(Message::Ping(msg_data)),
            OpCode::Pong => Ok(Message::Pong(msg_data)),
            OpCode::Close => {
                if msg_data.is_empty() {
                    Ok(Message::Close(None))
                } else {
                    let close =
                        CloseFrame::from_bytes(&msg_data).map_err(WebSocketError::CloseError)?;
                    Ok(Message::Close(Some(close)))
                }
            }
            OpCode::Continuation => {
                unreachable!("Continuation must not be possible because all the data is aggregated")
            }
        }
    }

    /// Sends a message.
    pub fn send(&mut self, message: impl Into<Message>) -> Result<(), WebSocketError> {
        let message = message.into();
        let op_code = OpCode::from_message(&message);
        let frame = Frame::builder(op_code).fin(true).data(message.into_bytes());

        self.send_frame(frame)
    }

    /// Send a ping to the client to check if still connected.
    ///
    /// # Returns
    /// An error if the client does not respond.
    pub fn ping(&mut self) -> Result<(), WebSocketError> {
        self.ping_timeout(None)
    }

    /// Sends a ping to check if the client still connected within the given timeout.
    ///
    /// # Returns
    /// An error if the client does not respond.
    pub fn ping_timeout(&mut self, timeout: Option<Duration>) -> Result<(), WebSocketError> {
        let id = Uuid::new_v4().to_simple_string().into_bytes();
        self.send(Message::Ping(id.clone()))?;

        match self.read(timeout)? {
            Message::Pong(bytes) => {
                if bytes != id {
                    return Err(WebSocketError::Other(
                        String::from("invalid pong received").into(),
                    ));
                }

                Ok(())
            }
            _ => Err(WebSocketError::Other(String::from("expected pong").into())),
        }
    }

    /// Sends a close signal to the client.
    pub fn close(mut self) -> Result<(), WebSocketError> {
        self.send(Message::Close(None))?;
        Ok(())
    }

    /// Sends a close signal to the client with the given code and reason.
    pub fn send_close(
        mut self,
        code: CloseStatusCode,
        reason: impl Into<String>,
    ) -> Result<(), WebSocketError> {
        let close = CloseFrame::new(code, reason.into());
        self.send(Message::Close(Some(close)))?;
        Ok(())
    }

    fn try_clone(&self) -> Option<Self> {
        let upgrade = self.upgrade.try_clone()?;
        let buffer_size = self.buf.len();
        let max_payload_length = self.max_payload_length;
        Some(WebSocket::with_config(
            upgrade,
            WebSocketConfig {
                buffer_size,
                max_payload_length,
            },
        ))
    }

    /// Split the websocket into a read and write part.
    pub fn split(self) -> Result<(WebSocketReader, WebSocketWriter), WebSocketSplitError> {
        let other = self.try_clone().ok_or(WebSocketSplitError)?;
        let read = WebSocketReader(self);
        let write = WebSocketWriter(other);

        Ok((read, write))
    }
}

impl WebSocket {
    fn read_exact(&mut self, bytes_len: usize) -> Result<Vec<u8>, WebSocketError> {
        let mut result = Vec::new();

        loop {
            let len = self.buf.len().min(bytes_len);
            let dst = &mut self.buf[..len];
            self.upgrade.read_exact(dst)?;

            result.extend_from_slice(dst);

            if result.len() >= bytes_len {
                break;
            }
        }

        Ok(result)
    }

    fn read_next_byte(&mut self) -> Result<u8, WebSocketError> {
        let mut buf = [0; 1];
        self.upgrade.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_frame_len(&mut self) -> Result<(bool, u64), WebSocketError> {
        let b1 = self.read_next_byte()?;
        let mask = (b1 & 0b1000_0000) != 0;
        let len_indicator = (b1 & 0b0111_1111) as u64;

        let len = if len_indicator < 125 {
            len_indicator
        } else if len_indicator == 126 {
            let b2 = self.read_next_byte()?;
            let b3 = self.read_next_byte()?;
            ((b2 as u64) << 8) | (b3 as u64)
        } else if len_indicator == 127 {
            let bytes = self.read_exact(8)?;
            (bytes[0] as u64) << 56
                | (bytes[1] as u64) << 48
                | (bytes[2] as u64) << 40
                | (bytes[3] as u64) << 32
                | (bytes[4] as u64) << 24
                | (bytes[5] as u64) << 16
                | (bytes[6] as u64) << 8
                | (bytes[7] as u64)
        } else {
            return Err(WebSocketError::InvalidPayloadLen(len_indicator));
        };

        if let Some(min) = self.max_payload_length {
            if (len as usize) > min {
                return Err(WebSocketError::PayloadTooBig {
                    min,
                    actual: len as usize,
                });
            }
        }

        Ok((mask, len))
    }

    fn read_masking_key(&mut self) -> Result<u32, WebSocketError> {
        let bytes = self.read_exact(4)?;
        let b0 = bytes[0] as u32;
        let b1 = bytes[1] as u32;
        let b2 = bytes[2] as u32;
        let b3 = bytes[3] as u32;

        Ok(b0 << 24 | b1 << 16 | b2 << 8 | b3)
    }

    fn read_payload(&mut self, mut len: u64, masking_key: u32) -> Result<Vec<u8>, WebSocketError> {
        let mut data = Vec::new();
        let masking_key_bytes = masking_key.to_be_bytes();

        while len > 0 {
            let bytes_read = self.upgrade.read(&mut self.buf)?;
            match bytes_read {
                0 => break,
                n => {
                    len -= n as u64;
                    let bytes = &mut self.buf[..n];

                    if masking_key != 0 {
                        bytes.iter_mut().enumerate().for_each(|(idx, b)| {
                            let mask_byte = masking_key_bytes[idx % 4];
                            *b ^= mask_byte;
                        });
                    }

                    data.extend_from_slice(bytes);
                }
            }
        }

        Ok(data)
    }

    /// Send a raw frame.
    pub fn send_frame(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        self.upgrade.write_all(&frame.into_bytes())?;
        Ok(())
    }

    /// Read a raw websocket frame.
    pub fn read_frame(&mut self) -> Result<Frame, WebSocketError> {
        // Specification: https://datatracker.ietf.org/doc/html/rfc6455#section-5.2

        // fin, rsv1, rsv2, rsv3, op_code
        let first_byte = self.read_next_byte()?;
        let fin = (first_byte & 0b1000_0000) != 0;
        let rsv1 = (first_byte & 0b0100_0000) != 0;
        let rsv2 = (first_byte & 0b0010_0000) != 0;
        let rsv3 = (first_byte & 0b0001_0000) != 0;
        let op_code_seq = first_byte & 0b0000_1111;
        let op_code = OpCode::from_byte(op_code_seq)
            .ok_or_else(|| WebSocketError::InvalidOpCode(op_code_seq))?;

        if rsv1 || rsv2 || rsv3 {
            log::warn!("rsv bits were set but extensions are not supported");
        }

        let (mask, payload_len) = self.read_frame_len()?;

        if !mask {
            return Err(WebSocketError::UnmaskedClientPayload);
        }

        let masking_key = if mask { self.read_masking_key()? } else { 0 };
        let payload = self.read_payload(payload_len, masking_key)?;

        Ok(Frame {
            fin,
            rsv1,
            rsv2,
            rsv3,
            op_code,
            mask,
            payload_len,
            masking_key,
            payload,
        })
    }
}

impl Debug for WebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocket").finish_non_exhaustive()
    }
}

impl Iterator for WebSocket {
    type Item = Result<Message, WebSocketError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.recv())
    }
}

pub struct WebSocketReader(WebSocket);
impl WebSocketReader {
    /// Reads a message.
    #[track_caller]
    pub fn recv(&mut self) -> Result<Message, WebSocketError> {
        self.0.recv()
    }

    #[track_caller]
    /// Reads a message and errors if timeout.
    pub fn recv_timeout(&mut self, timeout: Duration) -> Result<Message, WebSocketError> {
        self.0.recv_timeout(timeout)
    }
}

impl Iterator for WebSocketReader {
    type Item = Result<Message, WebSocketError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.recv())
    }
}

pub struct WebSocketWriter(WebSocket);
impl WebSocketWriter {
    /// Sends a message.
    pub fn send(&mut self, message: impl Into<Message>) -> Result<(), WebSocketError> {
        self.0.send(message)
    }

    /// Send a ping to the client to check if still connected.
    ///
    /// # Returns
    /// An error if the client does not respond.
    pub fn ping(&mut self) -> Result<(), WebSocketError> {
        self.0.ping()
    }

    /// Sends a ping to check if the client still connected within the given timeout.
    ///
    /// # Returns
    /// An error if the client does not respond.
    pub fn ping_timeout(&mut self, timeout: Option<Duration>) -> Result<(), WebSocketError> {
        self.0.ping_timeout(timeout)
    }
}
