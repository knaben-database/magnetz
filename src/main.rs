use anyhow::Result;
use elasticsearch::{
    http::{request::JsonBody, transport::Transport},
    BulkParts, Elasticsearch, MgetParts,
};
use lazy_static::lazy_static;
use redis::{self, Script};
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};
use tokio;

use config::Config;
use magnet::Magnet;
use scraper::TorrustTorrentStat;
use track_tracker::TrackTrackers;
use tracker::Tracker;

mod config;
mod es_requests;
mod magnet;
mod scraper;
mod track_tracker;
mod tracker;

lazy_static! {
    static ref CONFIG: Config = Config::new();
}


#[tokio::main]
async fn main() -> Result<()> {
    let redis_client = redis::Client::open(format!(
        "redis://{}:{}/",
        CONFIG.redis_host, CONFIG.redis_port
    ))?;
    let elasticsearch_transport = Transport::single_node(&format!(
        "http://{}:{}",
        CONFIG.elasticsearch_host, CONFIG.elasticsearch_port
    ))?;
    let elasticsearch_client = Elasticsearch::new(elasticsearch_transport);
    let mut con = redis_client.get_connection()?;

    // Lua script that gets 9 of the top heatmap values, deletes them and inserts a lock value with expiry of 7200 seconds
    let lock_expiry = CONFIG.lock_timer; // Lock expiry time in seconds
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
            .mget(MgetParts::Index(&CONFIG.elasticsearch_index))
            ._source_includes(&["MagnetUri"])
            .body(json!({"ids" : ids}))
            .send()
            .await?;

        let results = response
            .json::<es_requests::mget::ElasticsearchMgetResponse>()
            .await?;

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

        if magnet_results.is_empty() {
            println!("No magnets updated");
            continue;
        }

        // Clear the inserted magnets
        track_trackers.values_mut().for_each(|t| t.clear_magnets());

        let new_results: HashMap<String, (TorrustTorrentStat, u16)> = magnet_results
            .into_iter()
            .map(|(id, result)| {
                (
                    id,
                    (
                        TorrustTorrentStat {
                            leechers: result
                                .values()
                                .map(|stats| stats.leechers)
                                .max()
                                .unwrap_or(0),
                            seeders: result
                                .values()
                                .map(|stats| stats.seeders)
                                .max()
                                .unwrap_or(0),
                            completed: result
                                .values()
                                .map(|stats| stats.completed)
                                .max()
                                .unwrap_or(0),
                        },
                        result.len() as u16,
                    ),
                )
            })
            .collect();

        let mut body: Vec<JsonBody<Value>> = vec![];

        for (id, (magnet, tracker_count)) in new_results {
            println!("[{}] Trackers: {} {}", id, tracker_count, magnet);
            let bulk = es_requests::bulk::Bulk::new(magnet);
            body.push(json!({"update": {"_id": id}}).into());
            body.push(json!(bulk).into());
        }

        let response = elasticsearch_client
            .bulk(BulkParts::Index(&CONFIG.elasticsearch_index))
            .body(body)
            .request_timeout(Duration::from_secs(25))
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let Ok(response_json) = response.json::<Value>().await else {
            eprintln!("Failed to parse JSON");
            continue;
        };

        if response_json["errors"].as_bool().unwrap_or(false) == true {
            println!("Bulk had errors!");
        }

        let updated_magnets = response_json["items"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.get("update").and_then(|index| {
                            index
                                .get("result")
                                .and_then(|r| r.as_str())
                                .filter(|s| s == &"updated")
                        })
                    })
                    .count()
            })
            .unwrap_or(0);

        println!("Updated {} documents", updated_magnets);
    }
}
