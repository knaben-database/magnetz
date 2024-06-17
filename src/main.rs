use anyhow::Result;
use elasticsearch::{Elasticsearch, MgetParts};
use redis::{self, Commands, Script};
use serde_json::json;
use std::collections::HashMap;
use tokio;
use url::Url;

use elasticsearch_mget::ElasticsearchMgetResponse;

mod elasticsearch_mget;

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
    fn new(link: String) -> Magnet {
        let parsed = Url::parse(link.as_str()).expect(format!("Unparsable url {}", link).as_str());
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

    // Lua script that gets 9 of the top heatmap values, deletes them and inserts a lock value with expiry of 7200 seconds
    let lock_expiry = 7200; // Lock expiry time in seconds
    let top_n = 9; // Number of top IDs to retrieve
    let script = Script::new(&format!(r#"
        local result = redis.call('ZMPOP', 1, 'heatmap', 'MAX', 'COUNT', {top_n})
        local ids = result[2] or {{}}

        for i, id_score_pair in ipairs(ids) do
            local id = id_score_pair[1]
            local lock_key = 'lock:' .. id
            redis.call('SET', lock_key, '1')
            redis.call('EXPIRE', lock_key, {lock_expiry})
        end

        return ids
    "#));

    let heatmap: Vec<Vec<String>> = script.invoke(&mut con)?;
    let ids: Vec<String> = heatmap.into_iter().flat_map(|pair| pair.into_iter().take(1)).collect();

    let elasticsearch_client = Elasticsearch::default();
    let response = elasticsearch_client
        .mget(MgetParts::Index("torrent_cache2"))
        ._source_includes(&["MagnetUri"])
        .body(json!({"ids" : ids}))
        .send()
        .await?;

    let results = response.json::<ElasticsearchMgetResponse>().await?;

    let magnets: Vec<(String, Magnet)> = results
        .docs
        .into_iter()
        .filter(|d| d.source.magnet_uri.is_some())
        .map(|d| (d.id, Magnet::new(d.source.magnet_uri.unwrap())))
        .collect();

    println!("{:?}", magnets);

    Ok(())
}
