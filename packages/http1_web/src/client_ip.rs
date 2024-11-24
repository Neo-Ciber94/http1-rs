use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use http1::headers::{self, Headers};

use crate::{conn_info::ConnectionInfo, from_request::FromRequest, ErrorStatusCode};

#[derive(Debug)]
enum Inner {
    Ip(IpAddr),
    List(Vec<IpAddr>),
}

/// Enables to extract the client ip address.
#[derive(Debug)]
pub struct ClientIp(Inner);

impl ClientIp {
    /// Gets the client ip address.
    pub fn ip(&self) -> IpAddr {
        match &self.0 {
            Inner::Ip(ip_addr) => *ip_addr,
            Inner::List(vec) => vec[0],
        }
    }

    /// Get all the addresses for this client.
    pub fn addrs(&self) -> &[IpAddr] {
        match &self.0 {
            Inner::Ip(ip_addr) => std::slice::from_ref(ip_addr),
            Inner::List(vec) => vec.as_slice(),
        }
    }
}

impl FromRequest for ClientIp {
    type Rejection = ErrorStatusCode;

    fn from_request(
        req: &http1::request::Request<()>,
        extensions: &mut http1::extensions::Extensions,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        match get_ip_from_headers(req.headers()) {
            Some(inner) => Ok(ClientIp(inner)),
            None => {
                if let Ok(conn) = ConnectionInfo::<SocketAddr>::from_request(req, extensions, payload) {
                    return Ok(ClientIp(Inner::Ip(conn.ip())));
                }

                log::error!("Failed to retrieve the client ip");
                Err(ErrorStatusCode::InternalServerError)
            }
        }
    }
}

fn get_ip_from_headers(headers: &Headers) -> Option<Inner> {
    if headers.contains_key(headers::X_FORWARDED_FOR) {
        let addrs = headers
            .get_all(headers::X_FORWARDED_FOR)
            .filter_map(|s| IpAddr::from_str(s.as_str()).ok())
            .collect::<Vec<IpAddr>>();

        if !addrs.is_empty() {
            return Some(Inner::List(addrs));
        }
    }

    if let Some(addr) = headers
        .get(headers::X_REAL_IP)
        .and_then(|s| IpAddr::from_str(s.as_str()).ok())
    {
        return Some(Inner::Ip(addr));
    }

    if let Some(addr) = headers
        .get(headers::X_CLIENT_IP)
        .and_then(|s| IpAddr::from_str(s.as_str()).ok())
    {
        return Some(Inner::Ip(addr));
    }

    if headers.contains_key(headers::FORWARDED) {
        let mut addrs = Vec::new();

        for value in headers.get_all(headers::FORWARDED) {
            let str = value.as_str();

            if start_with_case_insensitive(str, "for=") {
                let raw = &str[4..];
                let maybe_ip: &str = if raw.starts_with("\"") && raw.ends_with("\"") {
                    &raw[1..raw.len() - 1]
                } else {
                    raw
                };

                match IpAddr::from_str(maybe_ip) {
                    Ok(x) => addrs.push(x),
                    Err(err) => {
                        log::debug!("Failed to parse Forwarded header ip: {err}")
                    }
                }
            }
        }

        if !addrs.is_empty() {
            return Some(Inner::List(addrs));
        }
    }

    None
}

fn start_with_case_insensitive(s: &str, prefix: &str) -> bool {
    let s_chars = s.chars();
    let prefix_chars = prefix.chars();

    // Compare the characters of the string and prefix
    s_chars
        .zip(prefix_chars)
        .all(|(s_char, p_char)| s_char.eq_ignore_ascii_case(&p_char))
}
