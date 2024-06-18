use super::tracker::Tracker;
use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use url::Url;

#[derive(Debug)]
pub struct Magnet {
    exact_topic: String,
    display_name: Option<String>,
    exact_length: Option<u64>,
    pub trackers: Vec<Tracker>,
}
impl Magnet {
    pub fn new(link: String) -> Result<Magnet> {
        let parsed = Url::parse(link.as_str())?;
        let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

        let exact_topic = params
            .get("xt")
            .with_context(|| "xt not found on magnet link")?
            .to_string();
        let display_name = params.get("dn").map(String::from);
        let exact_length = params.get("xl").and_then(|s| s.parse().ok());

        let mut trackers: Vec<Tracker> = Vec::new();
        for (key, value) in parsed.query_pairs() {
            if key == "tr" {
                if let Ok(tracker) = Tracker::new(&value) {
                    trackers.push(tracker);
                }
            }
        }

        Ok(Magnet {
            exact_topic,
            display_name,
            exact_length,
            trackers,
        })
    }
}
