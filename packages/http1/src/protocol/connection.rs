use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use super::h1::io::IoStream;

/// Represents a request connection stream to read the request and write the response.
pub enum Connection {
    Tcp(TcpStream),
    Io(IoStream),
}

impl Connection {
    pub fn from_io<T>(io: T) -> Self
    where
        T: Read + Write + Send + Sync + 'static,
    {
        Connection::Io(IoStream::new(io))
    }

    pub fn try_clone(&self) -> Option<Connection> {
        match self {
            Connection::Tcp(tcp_stream) => tcp_stream.try_clone().ok().map(Connection::Tcp),
            Connection::Io(io) => Some(Connection::Io(io.clone())),
        }
    }

    pub fn peer_addr(&self) -> Option<SocketAddr> {
        match self {
            Connection::Tcp(tcp_stream) => tcp_stream.peer_addr().ok(),
            Connection::Io(_) => None,
        }
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connection")
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Connection::Tcp(tcp_stream) => Read::read(tcp_stream, buf),
            Connection::Io(arc) => Read::read(arc, buf),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Connection::Tcp(tcp_stream) => Write::write(tcp_stream, buf),
            Connection::Io(arc) => Write::write(arc, buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Connection::Tcp(tcp_stream) => Write::flush(tcp_stream),
            Connection::Io(arc) => Write::flush(arc),
        }
    }
}

/// Request connection information after a request is accepted.
pub struct Connected {
    peer_addr: Option<SocketAddr>,
}

impl Connected {
    pub fn from_connection(conn: &Connection) -> Self {
        Connected {
            peer_addr: conn.peer_addr(),
        }
    }

    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }
}
