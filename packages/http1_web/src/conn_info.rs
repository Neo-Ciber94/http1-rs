use std::{net::IpAddr, str::FromStr};

use http1::headers;

use crate::{from_request::FromRequestRef, ErrorStatusCode};

/// Contains the ip address information of the request.
/// 
/// The address is check in the headers: `X-Forwarded-For`, `X-Real-IP`, `X-Client-IP` and `Forwarded`, if not found will be rejected,
/// it's recommended to use [`Option<ConnectionInfo>`] in case its not found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionInfo(Vec<IpAddr>);

impl ConnectionInfo {
    pub fn addrs(&self) -> &[IpAddr] {
        self.0.as_slice()
    }
}

impl FromRequestRef for ConnectionInfo {
    type Rejection = ErrorStatusCode;

    fn from_request_ref(
        req: &http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers();

        if headers.contains_key(headers::X_FORWARDED_FOR) {
            let addrs = headers
                .get_all(headers::X_FORWARDED_FOR)
                .filter_map(|s| IpAddr::from_str(s.as_str()).ok())
                .collect::<Vec<IpAddr>>();

            if !addrs.is_empty() {
                return Ok(ConnectionInfo(addrs));
            }
        }

        if let Some(addr) = headers
            .get(headers::X_REAL_IP)
            .and_then(|s| IpAddr::from_str(s.as_str()).ok())
        {
            return Ok(ConnectionInfo(vec![addr]));
        }

        if let Some(addr) = headers
            .get(headers::X_CLIENT_IP)
            .and_then(|s| IpAddr::from_str(s.as_str()).ok())
        {
            return Ok(ConnectionInfo(vec![addr]));
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
                        &raw
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
                return Ok(ConnectionInfo(addrs));
            }
        }

        log::error!("Failed to resolve connection info");
        Err(ErrorStatusCode::InternalServerError)
    }
}

fn start_with_case_insensitive(s: &str, prefix: &str) -> bool {
    let s_chars = s.chars();
    let prefix_chars = prefix.chars();

    // Compare the characters of the string and prefix
    s_chars
        .zip(prefix_chars)
        .all(|(s_char, p_char)| s_char.eq_ignore_ascii_case(&p_char))
}
