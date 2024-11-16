use std::{fmt::Display, string::FromUtf8Error};

use super::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
    pub fn from_message(msg: &Message) -> Self {
        match msg {
            Message::Binary(_) => OpCode::Binary,
            Message::Text(_) => OpCode::Text,
            Message::Ping(_) => OpCode::Ping,
            Message::Pong(_) => OpCode::Pong,
            Message::Close(_) => OpCode::Close,
        }
    }

    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(OpCode::Continuation),
            0x1 => Some(OpCode::Text),
            0x2 => Some(OpCode::Binary),
            0x8 => Some(OpCode::Close),
            0x9 => Some(OpCode::Ping),
            0xA => Some(OpCode::Pong),
            _ => None,
        }
    }

    pub fn as_bit(&self) -> u8 {
        *self as u8
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub op_code: OpCode,
    pub mask: bool,
    pub payload_len: u64,
    pub masking_key: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn builder(op_code: OpCode) -> FrameBuilder {
        FrameBuilder::new(op_code)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let Frame {
            fin,
            rsv1,
            rsv2,
            rsv3,
            op_code,
            mask,
            payload_len,
            masking_key,
            payload,
        } = self;

        let mut bytes = Vec::new();

        // fin, rsv1, rsv2, rsv3, op_code
        let b_fin = (fin as u8) << 7;
        let b_rsv1 = (rsv1 as u8) << 6;
        let b_rsv2 = (rsv2 as u8) << 5;
        let b_rsv3 = (rsv3 as u8) << 4;
        let b0 = b_fin | b_rsv1 | b_rsv2 | b_rsv3 | op_code.as_bit();

        bytes.push(b0);

        // mask and payload len
        let b_mask = (mask as u8) << 7;
        match payload_len {
            n if n < 126 => {
                let b1 = b_mask | payload_len as u8;
                bytes.push(b1);
            }
            n if n < u16::MAX as u64 => {
                let b1 = b_mask | 126;
                let b2 = (n as u16).to_be_bytes();
                bytes.push(b1);
                bytes.extend_from_slice(&b2);
            }
            n => {
                let b1 = b_mask | 127;
                let b2 = n.to_be_bytes();
                bytes.push(b1);
                bytes.extend_from_slice(&b2);
            }
        };

        // masking key
        if mask {
            bytes.extend_from_slice(&masking_key.to_be_bytes());
        }

        // payload
        bytes.extend(payload);

        bytes
    }
}

/// Allow to construct a raw frame.
#[derive(Debug)]
pub struct FrameBuilder(Frame);

#[allow(dead_code)]
impl FrameBuilder {
    pub fn new(op_code: OpCode) -> Self {
        FrameBuilder(Frame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            op_code,
            mask: false,
            payload_len: 0,
            masking_key: 0,
            payload: Vec::new(),
        })
    }

    pub fn fin(mut self, is_fin: bool) -> Self {
        self.0.fin = is_fin;
        self
    }

    pub fn rsv(mut self, rsv1: bool, rsv2: bool, rsv3: bool) -> Self {
        self.0.rsv1 = rsv1;
        self.0.rsv2 = rsv2;
        self.0.rsv3 = rsv3;
        self
    }

    pub fn mask(mut self, masking_key: Option<u32>) -> Self {
        match masking_key {
            Some(key) => {
                self.0.mask = false;
                self.0.masking_key = key;
            }
            None => {
                self.0.mask = false;
                self.0.masking_key = 0;
            }
        }

        self
    }

    pub fn generate_mask(self) -> Self {
        let key = rng::random::<u32>();
        self.mask(Some(key))
    }

    pub fn data(mut self, bytes: Vec<u8>) -> Frame {
        self.0.payload_len = bytes.len() as u64;
        self.0.payload = bytes;
        self.0
    }
}

#[derive(Debug)]
pub enum CloseFrameError {
    CloseFrameTooShort,
    InvalidStatusCode(u16),
    Utf8(FromUtf8Error),
}

impl Display for CloseFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloseFrameError::CloseFrameTooShort => write!(f, "close frame is too short"),
            CloseFrameError::InvalidStatusCode(code) => write!(f, "invalid close code: `{code}`"),
            CloseFrameError::Utf8(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct CloseFrame {
    code: CloseStatusCode,
    reason: String,
}

impl CloseFrame {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CloseFrameError> {
        if bytes.len() < 2 {
            return Err(CloseFrameError::CloseFrameTooShort);
        }

        let code_raw = u16::from_be_bytes([bytes[0], bytes[1]]);
        let code = CloseStatusCode::from_u16(code_raw)
            .ok_or(CloseFrameError::InvalidStatusCode(code_raw))?;

        let reason = String::from_utf8(bytes[3..].to_vec()).map_err(CloseFrameError::Utf8)?;

        Ok(CloseFrame { code, reason })
    }

    pub fn new(code: CloseStatusCode, reason: impl Into<String>) -> Self {
        CloseFrame {
            code,
            reason: reason.into(),
        }
    }

    pub fn code(&self) -> CloseStatusCode {
        self.code
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn into_string(self) -> String {
        self.reason
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let b1 = self.code().as_u16().to_be_bytes();
        let b2 = self.reason.as_bytes();

        let mut b = Vec::new();
        b.extend_from_slice(&b1);
        b.extend_from_slice(b2);
        b
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CloseStatusCode {
    NormalClosure = 1000,
    GoingAway = 1001,
    ProtocolError = 1002,
    UnsupportedData = 1003,
    NoStatusRcvd = 1005,
    AbnormalClosure = 1006,
    InvalidFramePayloadData = 1007,
    PolicyViolation = 1008,
    MessageTooBig = 1009,
    MandatoryExt = 1010,
    InternalServerError = 1011,
    TlsHandshake = 1015,
}

impl CloseStatusCode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1000 => Some(CloseStatusCode::NormalClosure),
            1001 => Some(CloseStatusCode::GoingAway),
            1002 => Some(CloseStatusCode::ProtocolError),
            1003 => Some(CloseStatusCode::UnsupportedData),
            1005 => Some(CloseStatusCode::NoStatusRcvd),
            1006 => Some(CloseStatusCode::AbnormalClosure),
            1007 => Some(CloseStatusCode::InvalidFramePayloadData),
            1008 => Some(CloseStatusCode::PolicyViolation),
            1009 => Some(CloseStatusCode::MessageTooBig),
            1010 => Some(CloseStatusCode::MandatoryExt),
            1011 => Some(CloseStatusCode::InternalServerError),
            1015 => Some(CloseStatusCode::TlsHandshake),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}
