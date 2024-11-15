use std::io::Read;

use http1::{error::BoxError, protocol::upgrade::Upgrade};

const BUFFER_SIZE: usize = 4 * 1024;

#[derive(Clone)]
pub struct WebSocket {
    upgrade: Upgrade,
    buf: Box<[u8]>,
}

impl WebSocket {
    pub fn new(upgrade: Upgrade) -> Self {
        let buf = vec![0; BUFFER_SIZE].into_boxed_slice();
        WebSocket { upgrade, buf }
    }

    fn read_exact(&mut self, bytes_len: usize) -> Result<Vec<u8>, BoxError> {
        let mut result = Vec::new();

        loop {
            let len = self.buf.len().min(bytes_len);
            let mut dst = &mut self.buf[..len];
            self.upgrade.read_exact(&mut dst)?;

            result.extend_from_slice(dst);

            if result.len() >= bytes_len {
                break;
            }
        }

        Ok(result)
    }

    fn read_next_byte(&mut self) -> Result<u8, BoxError> {
        let mut buf = [0; 1];
        self.upgrade.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_frame_len(&mut self) -> Result<(bool, u64), BoxError> {
        let b1 = self.read_next_byte()?;
        let mask = (b1 & 0b1000_0000) != 0;
        let len_indicator = (b1 & 0b0111_1111) as u64;

        if len_indicator < 125 {
            Ok((mask, len_indicator))
        } else if len_indicator == 126 {
            let b2 = self.read_next_byte()?;
            let b3 = self.read_next_byte()?;
            let len = ((b2 as u64) << 8) | (b3 as u64);
            Ok((mask, len))
        } else if len_indicator == 127 {
            let bytes = self.read_exact(8)?;
            let len = (bytes[0] as u64) << 56
                | (bytes[1] as u64) << 48
                | (bytes[2] as u64) << 40
                | (bytes[3] as u64) << 32
                | (bytes[4] as u64) << 24
                | (bytes[5] as u64) << 16
                | (bytes[6] as u64) << 8
                | (bytes[7] as u64);

            Ok((mask, len))
        } else {
            Err(format!("Invalid payload len: `{len_indicator}`").into())
        }
    }

    fn read_masking_key(&mut self) -> Result<u32, BoxError> {
        let bytes = self.read_exact(4)?;
        let b0 = bytes[0] as u32;
        let b1 = bytes[1] as u32;
        let b2 = bytes[2] as u32;
        let b3 = bytes[3] as u32;

        Ok(b0 << 24 | b1 << 16 | b2 << 8 | b3)
    }

    fn read_payload(&mut self, mut len: u64, masking_key: u32) -> Result<Vec<u8>, BoxError> {
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
                            *b = *b ^ mask_byte;
                        });
                    }

                    data.extend_from_slice(bytes);
                }
            }
        }

        Ok(data)
    }

    fn read_frame(&mut self) -> Result<Frame, BoxError> {
        // fin, rsv1, rsv2, rsv3, op_code
        let first_byte = self.read_next_byte()?;
        let fin = first_byte & 0b0111_1111 == 1;
        let rsv1 = first_byte & 0b1011_1111;
        let rsv2 = first_byte & 0b1101_1111;
        let rsv3 = first_byte & 0b1110_1111;
        let op_code_seq = first_byte << 4 & 0b1111_0000;
        let op_code = OpCode::from_byte(op_code_seq)
            .ok_or_else(|| format!("Invalid op_code `{op_code_seq}`"))?;

        let (mask, payload_len) = self.read_frame_len()?;

        if !mask {
            return Err(String::from("client to server payload should always be encoded").into());
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

    pub fn read(&mut self) -> Result<Message, BoxError> {
        let mut msg_data = Vec::new();
        let mut msg_op_code: Option<OpCode> = None;

        loop {
            let Frame {
                fin,
                op_code,
                payload,
                ..
            } = self.read_frame()?;

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
                let text = String::from_utf8(msg_data)?;
                Ok(Message::Text(text))
            }
            OpCode::Ping => Ok(Message::Ping(msg_data)),
            OpCode::Pong => Ok(Message::Pong(msg_data)),
            OpCode::Close => Ok(Message::Close),
            OpCode::Continuation => {
                unreachable!("Continuation must not be possible because all the data is aggregated")
            }
        }
    }

    pub fn send(&mut self, message: Message) -> Result<(), BoxError> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
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

    pub fn to_bit(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

#[derive(Debug)]
struct Frame {
    fin: bool,
    rsv1: u8,
    rsv2: u8,
    rsv3: u8,
    op_code: OpCode,
    mask: bool,
    payload_len: u64,
    masking_key: u32,
    payload: Vec<u8>,
}

impl Frame {
    pub fn builder(op_code: OpCode) -> FrameBuilder {
        FrameBuilder::new(op_code)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        // fin, rsv1, rsv2, rsv3, op_code
        todo!()
    }
}

#[derive(Debug)]
struct FrameBuilder(Frame);
impl FrameBuilder {
    pub fn new(op_code: OpCode) -> Self {
        FrameBuilder(Frame {
            fin: true,
            rsv1: 0,
            rsv2: 0,
            rsv3: 0,
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

    pub fn rsv(mut self, rsv1: u8, rsv2: u8, rsv3: u8) -> Self {
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
