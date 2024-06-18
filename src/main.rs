use anyhow::{bail, Context, Result};
use elasticsearch::{Elasticsearch, MgetParts};
use redis::{self, Commands, Script};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use tokio;
use url::Url;

use elasticsearch_mget::ElasticsearchMgetResponse;
use tracker::Tracker;
use magnet::Magnet;

mod elasticsearch_mget;
mod tracker;
mod magnet;

struct Config {
    elasticsearch_host: String,
    elasticsearch_port: u16,
    elasticsearch_index: String,
    redis_host: String,
    redis_port: u16,
    redis_key: String,
}



#[tokio::main]
async fn main() -> Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let elasticsearch_client = Elasticsearch::default();
    let mut con = redis_client.get_connection()?;

    // Lua script that gets 9 of the top heatmap values, deletes them and inserts a lock value with expiry of 7200 seconds
    let lock_expiry = 7200; // Lock expiry time in seconds
    let top_n = 9; // Number of top IDs to retrieve
    let script = Script::new(&format!(
        r#"
        local result = redis.call('ZMPOP', 1, 'heatmap', 'MAX', 'COUNT', {top_n})
        local ids = result[2] or {{}}

        for i, id_score_pair in ipairs(ids) do
            local id = id_score_pair[1]
            local lock_key = 'lock:' .. id
            redis.call('SET', lock_key, '1')
            redis.call('EXPIRE', lock_key, {lock_expiry})
        end

        return ids
    "#
    ));
    let mut trackers: HashSet<Tracker> = HashSet::new();

    let heatmap: Vec<Vec<String>> = script.invoke(&mut con)?;
    let ids: Vec<String> = heatmap
        .into_iter()
        .flat_map(|pair| pair.into_iter().take(1))
        .collect();

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
        .filter_map(|d| d.source.magnet_uri.and_then(|mu| Some((d.id, mu))))
        .filter_map(|(id, mu)| Magnet::new(mu).ok().map(|magnet| (id, magnet)))
        .collect();

    for (id, magnet) in magnets {
        for tracker in magnet.trackers {
            // continue here
            trackers.insert(tracker);
        }
    }

    println!("Trackers: {:?}", trackers);

    Ok(())
}
