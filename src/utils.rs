use core::fmt;

#[derive(Clone)]
///Parsed semi valid url limited in length to `u16::MAX` bytes
///
///Display implementation will hide password for security purposes
pub struct Url {
    inner: compact_str::CompactString,
    schema_end: u16,
    host_start: u16,
    port_start: u16,
    path_start: u16,
    user_start: u16,
    pass_start: u16,
}

impl Url {
    const SCHEMA_SEP: &str = "://";
    const PATH_SEP: char = '/';
    const AUTH_SEP: char = '@';

    ///Fails if url is not starting with ASCII alpha character or too big.
    pub fn parse(inner: compact_str::CompactString) -> Option<Self> {
        let value = inner.as_str();
        if !value.starts_with(|ch: char| ch.is_ascii_alphabetic()) || value.len() > u16::MAX as usize {
            return None;
        }

        let path_start;
        let mut user_start = 0;
        let mut pass_start = 0;
        let mut host_start = 0;
        let mut port_start = 0;
        let mut schema_end = 0;

        let mut value_parts = value.splitn(2, Self::SCHEMA_SEP);
        let schema_or_rest = match value_parts.next() {
            Some(value) => value,
            None => unreach!()
        };
        let rest = match value_parts.next() {
            Some(rest) => {
                schema_end = schema_or_rest.len() as u16;
                host_start = schema_end + Self::SCHEMA_SEP.len() as u16;
                rest
            },
            None => schema_or_rest
        };

        let mut host = if let Some((host, _path)) = rest.split_once(Self::PATH_SEP) {
            path_start = host_start + host.len() as u16;
            host
        } else {
            path_start = value.len() as _;
            rest
        };

        if let Some((auth, real_host)) = host.split_once(Self::AUTH_SEP) {
            user_start = host_start;
            if let Some((user, _pass)) = auth.split_once(':') {
                pass_start = user_start + user.len() as u16 + 1
            }
            host_start += auth.len() as u16 + 1;
            host = real_host;
        }

        if let Some((host, _port)) = host.split_once(':') {
            port_start = host_start + host.len() as u16 + 1;
        }

        Some(Self {
            inner,
            schema_end,
            host_start,
            port_start,
            path_start,
            user_start,
            pass_start,
        })
    }

    #[inline(always)]
    pub fn schema(&self) -> Option<&str> {
        if self.schema_end == 0 {
            None
        } else {
            Some(&self.inner[..self.schema_end as usize])
        }
    }

    #[inline(always)]
    pub fn username(&self) -> Option<&str> {
        if self.user_start == 0 {
            None
        } else {
            let end = match self.pass_start {
                0 => self.host_start - 1,
                pass_start => pass_start - 1,
            };
            Some(&self.inner[self.user_start as usize..end as usize])
        }
    }

    #[inline(always)]
    pub fn password(&self) -> Option<&str> {
        if self.pass_start == 0 {
            None
        } else {
            Some(&self.inner[self.pass_start as usize..self.host_start as usize-1])
        }
    }

    #[inline(always)]
    pub fn host(&self) -> &str {
        let end = match self.port_start {
            0 => self.path_start,
            port_start => port_start - 1,
        };
        &self.inner[self.host_start as usize..end as usize]
    }

    #[inline(always)]
    pub fn host_port(&self) -> &str {
        &self.inner[self.host_start as usize..self.path_start as usize]
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        &self.inner[self.path_start as usize..]
    }

    #[inline(always)]
    pub fn port(&self) -> Option<&str> {
        if self.port_start == 0 {
            None
        } else {
            Some(&self.inner[self.port_start as usize..self.path_start as usize])
        }
    }

    #[inline(always)]
    pub fn port_u16(&self) -> Option<u16> {
        self.port().and_then(|port| port.parse().ok())
    }
}

impl fmt::Debug for Url {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(schema) = self.schema() {
            fmt.write_str(schema)?;
            fmt.write_str(Self::SCHEMA_SEP)?;
        }

        if let Some(user) = self.username() {
            fmt.write_str(user)?;
            if let Some(password) = self.password() {
                fmt.write_str(":")?;
                for _ in 0..password.len() {
                    fmt.write_str("*")?;
                }
            }
            fmt.write_str("@")?;
        }

        fmt.write_str(self.host_port())?;
        fmt.write_str(self.path())
    }
}

impl core::str::FromStr for Url {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match Url::parse(compact_str::CompactString::new(value.trim())) {
            Some(url) => Ok(url),
            None => Err(())
        }
    }
}
