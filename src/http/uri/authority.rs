use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Authority {
    user_info: Option<String>,
    host: String,
    port: Option<u16>,
}

impl Authority {
    pub fn new(user_info: Option<String>, host: String, port: Option<u16>) -> Self {
        Authority {
            user_info,
            host,
            port,
        }
    }

    pub fn user_info(&self) -> Option<&str> {
        self.user_info.as_deref()
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user_info) = &self.user_info {
            write!(f, "{user_info}")?;
        }

        write!(f, "{}", self.host)?;

        if let Some(port) = &self.port {
            write!(f, ":{}", port)?;
        }

        Ok(())
    }
}
