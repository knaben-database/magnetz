use anyhow::Result;
use elasticsearch::{Elasticsearch, MgetParts};
use redis::{self, Script};
use serde_json::json;
use std::collections::HashMap;
use tokio;
use track_tracker::TrackTrackers;

use elasticsearch_mget::ElasticsearchMgetResponse;
use magnet::Magnet;
use tracker::Tracker;

use scraper::Scraper;

mod elasticsearch_mget;
mod magnet;
mod track_tracker;
mod tracker;
mod scraper;

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
    let top_n = 49; // Number of top IDs to retrieve
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

    let mut track_trackers: HashMap<Tracker, TrackTrackers> = HashMap::new();
    for (id, magnet) in magnets.clone() {
        for magnet_tracker in magnet.clone().get_trackers() {
            let entry = track_trackers.entry(magnet_tracker.clone()).or_insert_with(|| TrackTrackers::new(magnet_tracker));
            entry.insert_magnet((id.clone(), magnet.clone()));
        }
    }

    for track_tracker in track_trackers.values_mut() {
        track_tracker.scrape_all();
    }

return     Ok(());

    let mut trackers: HashMap<Tracker, HashMap<String, Magnet>> = HashMap::new();
    for (id, magnet) in magnets.clone() {
        for magnet_tracker in magnet.clone().get_trackers() {
            let entry = trackers.entry(magnet_tracker).or_insert_with(HashMap::new);
            entry.insert(id.clone(), magnet.clone());
        }
    }

    for (tracker, magnets) in trackers {
        println!("Tracker: {}", tracker);
        for (id, magnet) in magnets {
            let mut stuff = magnet.to_string();
            stuff.truncate(150);
            println!("  {}", stuff);
        }
    }

    Ok(())
}
