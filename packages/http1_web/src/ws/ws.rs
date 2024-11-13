use http1::protocol::upgrade::Upgrade;


pub struct WebSocket(Upgrade);

impl WebSocket {
    pub fn new(upgrade: Upgrade) -> Self {
        WebSocket(upgrade)
    }
}