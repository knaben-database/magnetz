use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use url::Url;

use super::Magnet;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
enum TrackerSchema {
    UDP,
    HTTP,
    HTTPS,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct Tracker {
    host: String,
    port: u16,
    schema: TrackerSchema,
}
impl Tracker {
    pub fn new(link: &str) -> Result<Tracker> {
        let url = Url::parse(link)?;
        let host = url
            .host_str()
            .with_context(|| format!("Host is not valid in link: {}", link))?
            .to_string();
        let port = url
            .port_or_known_default()
            .with_context(|| format!("No valid port found for link: {}", link))?;

        let schema = match url.scheme() {
            "https" => TrackerSchema::HTTPS,
            "http" => TrackerSchema::HTTP,
            "udp" => TrackerSchema::UDP,
            s => bail!("Unsupported schema: {}", s),
        };

        Ok(Tracker { host, port, schema })
    }
}
impl fmt::Display for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema = match self.schema {
            TrackerSchema::UDP => "udp",
            TrackerSchema::HTTP => "http",
            TrackerSchema::HTTPS => "https",
        };
        write!(f, "{}://{}:{}", schema, self.host, self.port)
    }
}
