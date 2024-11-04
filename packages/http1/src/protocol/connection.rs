use std::{fs::File, net::TcpStream};

/// Represents a request connection stream to read the request and write the response.
pub trait Connection: std::io::Write + std::io::Read + Sized {
    type Err: std::error::Error;

    /// Try cloning this connection stream.
    fn try_clone(&self) -> Result<Self, Self::Err>;
}

impl Connection for TcpStream {
    type Err = std::io::Error;

    fn try_clone(&self) -> Result<Self, Self::Err> {
        self.try_clone()
    }
}

impl Connection for File {
    type Err = std::io::Error;

    fn try_clone(&self) -> Result<Self, Self::Err> {
        File::try_clone(&self)
    }
}
