use anyhow::Result;
use serde_json::{json, Value};
use elasticsearch::Elasticsearch;
use std::collections::HashMap;
use url::Url;
use tokio;
use redis::{self, Commands};

struct Config {
    elasticsearch_host: String,
    elasticsearch_port: u16,
    elasticsearch_index: String,
    redis_host: String,
    redis_port: u16,
    redis_key: String,
}

#[derive(Debug)]
struct Magnet {
    exact_topic: String,
    display_name: Option<String>,
    exact_length: Option<u64>,
    address_trackers: Vec<String>,
}
impl Magnet {
    fn new(link: &str) -> Magnet {
        let parsed = Url::parse(link).expect(format!("Unparsable url {}", link).as_str());
        let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

        let exact_topic = params
            .get("xt")
            .expect("Required field xt not found")
            .to_string();
        let display_name = params.get("dn").map(String::from);
        let exact_length = params.get("xl").and_then(|s| s.parse().ok());

        let mut address_trackers = Vec::new();
        for (key, value) in parsed.query_pairs() {
            if key == "tr" {
                address_trackers.push(value.into_owned());
            }
        }

        Magnet {
            exact_topic,
            display_name,
            exact_length,
            address_trackers,
        }
    }

    fn get_info(&self) {}
}

enum TrackerType {
    UDP,
    HTTP,
}

struct Tracker {
    address: String,
    port: u16,
    tracker_type: TrackerType,
}
impl Tracker {
    fn new(link: &str) -> Tracker {
        Tracker {
            address: "wow".to_string(),
            port: 123,
            tracker_type: TrackerType::UDP,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = redis_client.get_connection()?;

    let elasticsearch_client = Elasticsearch::default();
    let something = elasticsearch_client.mget(elasticsearch::MgetParts::Index("torrent_cache2")).body(
        json!({
          "ids" : ["e82aee90213ca36e14be64c369642f824b2532cb", "05bc3934750437d87a6a148a92a1a59ab6ae8db2"]
        })
    ).send().await?;

    println!("{}", something.text().await?);
    return Ok(());

    let heatmap: Vec<(String, u32)> = con.zrevrange_withscores("heatmap", 0, 99)?;
    for score in heatmap.clone() {
        let (id, score) = score;
    }

    println!("{:?}", heatmap);

    Ok(())
}
