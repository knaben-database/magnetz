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

use scraper::TorrustTorrentStat;

mod elasticsearch_mget;
mod magnet;
mod scraper;
mod track_tracker;
mod tracker;

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

    let mut track_trackers: HashMap<Tracker, TrackTrackers> = HashMap::new();

    loop {
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
            .filter(|(_, magnet)| magnet.info_hash.len() == 40) // Only sha1 supported
            .collect();

        for (id, magnet) in magnets.clone() {
            for magnet_tracker in magnet.clone().get_trackers() {
                let entry = track_trackers
                    .entry(magnet_tracker.clone())
                    .or_insert_with(|| TrackTrackers::new(magnet_tracker));
                entry.insert_magnet((id.clone(), magnet.clone()));
            }
        }

        let mut magnet_results: HashMap<String, HashMap<String, TorrustTorrentStat>> =
            HashMap::new();
        for track_tracker in track_trackers.values_mut() {
            match track_tracker.scrape_all() {
                Ok(r) => {
                    for (id, magnet) in r {
                        let es_entry = magnet_results.entry(id).or_insert_with(HashMap::new);
                        es_entry.insert(track_tracker.tracker.to_string(), magnet);
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }

        for (id, result) in magnet_results {
            println!("{}: ", id);
            for (tracker_name, stats) in result {
                println!("  {} -> {}", tracker_name, stats);
            }
        }

        // Clear the inserted magnets
        track_trackers.values_mut().for_each(|t| t.clear_magnets())
    }
}
