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

    fn read_frame_len(&mut self) -> Result<(bool, u128), BoxError> {
        let mut len = 0;

        let b1 = self.read_next_byte()?;
        let mask = b1 & 0b1000_0000 == 1;
        len = ((b1 as u128) << 1) & 0x1111_1110;

        if len < 125 {
            Ok((mask, len))
        } else if len == 126 {
            let b2 = self.read_next_byte()?;
            let b3 = self.read_next_byte()?;
            len = b1 as u128 & (b2 as u128 >> 8) & (b3 as u128 >> 16);
            Ok((mask, len))
        } else if len == 127 {
            let bytes = self.read_exact(8)?;
            Ok((mask, len))
        } else {
            Err(format!("Invalid payload len: `{len}`").into())
        }
    }

    fn read_masking_key(&mut self) -> Result<u32, BoxError> {
        todo!()
    }

    fn read_payload(&mut self, len: u128, masking_key: u32) -> Result<Vec<u8>, BoxError> {
        todo!()
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

        todo!()
        // let payload_len = self.read_frame_len()?;
        // let masking_key = self.read_masking_key()?;
        // let payload = self.read_payload(payload_len, masking_key)?;

        // Ok(Frame {
        //     fin,
        //     rsv1,
        //     rsv2,
        //     rsv3,
        //     op_code,
        //     mask,
        //     payload_len,
        //     masking_key,
        //     payload,
        // })
    }

    // pub fn read(&mut self) -> Result<Message, BoxError> {
    //     let mut bytes = Vec::new();

    //     loop {
    //         let read_bytes = self.upgrade.read(&mut self.buf)?;
    //         match read_bytes {
    //             0 => {}
    //             n => {}
    //         }
    //     }

    //     todo!()
    // }

    pub fn send(&mut self, message: Message) -> Result<(), BoxError> {
        todo!()
    }
}

#[derive(PartialEq, Eq)]
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
}

struct Frame {
    fin: bool,
    rsv1: u8,
    rsv2: u8,
    rsv3: u8,
    op_code: OpCode,
    mask: bool,
    payload_len: u128,
    masking_key: u32,
    payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}
