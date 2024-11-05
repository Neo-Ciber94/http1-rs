use std::{
    fs::File,
    net::{SocketAddr, TcpStream},
};

/// Represents a request connection stream to read the request and write the response.
pub trait Connection: std::io::Write + std::io::Read + Sized {
    type Err: std::error::Error;

    /// Try cloning this connection stream to get the write and read part.
    fn try_clone(&self) -> Result<Self, Self::Err>;

    /// Returns the ip address of this connection.
    fn peer_addr(&self) -> Option<SocketAddr>;
}

impl Connection for TcpStream {
    type Err = std::io::Error;

    fn try_clone(&self) -> Result<Self, Self::Err> {
        TcpStream::try_clone(&self)
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr().ok()
    }
}

impl Connection for File {
    type Err = std::io::Error;

    fn try_clone(&self) -> Result<Self, Self::Err> {
        File::try_clone(&self)
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        None
    }
}

impl<T: Connection> Connection for Box<T> {
    type Err = T::Err;

    fn try_clone(&self) -> Result<Self, Self::Err> {
        (&**self).try_clone().map(Box::new)
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        (&**self).peer_addr()
    }
}


/// Request connection information after a request is accepted.
pub struct Connected { 
    peer_addr: Option<SocketAddr>
}

impl Connected {
    pub fn from_connection<C: Connection>(conn: &C) -> Self {
        Connected {
            peer_addr: conn.peer_addr()
        }
    }

    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }
}