use anyhow::{Context, Result};
use std::{collections::HashMap, fmt};
use url::Url;

use super::tracker::Tracker;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Magnet {
    link: Url,
    exact_topic: String,
    display_name: Option<String>,
    exact_length: Option<u64>,
}
impl Magnet {
    pub fn new(link: String) -> Result<Magnet> {
        let parsed_link = Url::parse(link.as_str())?;
        let params: HashMap<_, _> = parsed_link.query_pairs().into_owned().collect();

        let exact_topic = params
            .get("xt")
            .with_context(|| "xt not found on magnet link")?
            .to_string();
        let display_name = params.get("dn").map(String::from);
        let exact_length = params.get("xl").and_then(|s| s.parse().ok());

        Ok(Magnet {
            link: parsed_link,
            exact_topic,
            display_name,
            exact_length,
        })
    }

    pub fn get_trackers(self) -> Vec<Tracker> {
        self.link
            .query_pairs()
            .filter(|(key, _)| key == "tr")
            .filter_map(|(_, value)| Tracker::new(&value).ok())
            .collect()
    }
}
impl fmt::Display for Magnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.link.as_str())
    }
}
