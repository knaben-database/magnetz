use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use url::Url;

use super::Magnet;

#[derive(PartialEq, Debug, Hash)]
enum TrackerSchema {
    UDP,
    HTTP,
    HTTPS,
}

#[derive(Debug)]
pub struct Tracker {
    host: String,
    port: u16,
    schema: TrackerSchema,
    magnets: HashSet<Magnet>,
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
        let magnets = HashSet::new();

        Ok(Tracker {
            host,
            port,
            schema,
            magnets,
        })
    }
}
impl PartialEq for Tracker {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port && self.schema == other.schema
    }
}
impl Hash for Tracker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.host.hash(state);
        self.port.hash(state);
        self.schema.hash(state);
    }
}
impl Eq for Tracker {}
