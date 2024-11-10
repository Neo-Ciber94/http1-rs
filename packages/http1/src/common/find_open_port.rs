use std::{fmt::Debug, net::TcpListener, ops::RangeBounds};

pub fn find_open_port_in_range<T>(range: T) -> std::io::Result<u16>
where
    T: RangeBounds<u16> + Debug,
{
    let start = match range.start_bound().cloned() {
        std::ops::Bound::Included(x) => x,
        std::ops::Bound::Excluded(x) => x + 1,
        std::ops::Bound::Unbounded => u16::MIN,
    };

    let end = match range.end_bound().cloned() {
        std::ops::Bound::Included(x) => x + 1,
        std::ops::Bound::Excluded(x) => x,
        std::ops::Bound::Unbounded => u16::MAX,
    };

    for port_num in start..end {
        if TcpListener::bind(("127.0.0.1", port_num)).is_ok() {
            return Ok(port_num);
        }
    }

    Err(std::io::Error::other(format!(
        "no available ports in the range: {range:?}"
    )))
}

pub fn find_open_port() -> std::io::Result<u16> {
    find_open_port_in_range(3000..)
}
