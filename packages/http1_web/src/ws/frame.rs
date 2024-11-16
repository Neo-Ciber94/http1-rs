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
            Message::Close => OpCode::Close,
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
#[derive(Debug)]
pub struct Frame {
    pub(crate) fin: bool,
    pub(crate) rsv1: bool,
    pub(crate) rsv2: bool,
    pub(crate) rsv3: bool,
    pub(crate) op_code: OpCode,
    pub(crate) mask: bool,
    pub(crate) payload_len: u64,
    pub(crate) masking_key: u32,
    pub(crate) payload: Vec<u8>,
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

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.0.payload_len = payload.len() as u64;
        self.0.payload = payload;
        self
    }

    pub fn push_payload(mut self, payload: &[u8]) -> Self {
        self.0.payload_len += payload.len() as u64;
        self.0.payload.extend_from_slice(payload);
        self
    }

    pub fn build(self) -> Frame {
        self.0
    }
}
