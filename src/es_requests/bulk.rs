use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::scraper::TorrustTorrentStat;

#[derive(Serialize, Deserialize)]
struct Entry {
    #[serde(rename = "LiveSeeders")]
    live_seeders: u64,
    #[serde(rename = "LivePeers")]
    live_peers: u64,
    #[serde(rename = "LiveGrabs")]
    live_grabs: u64,
    #[serde(rename = "CheckedMagnet")]
    checked_magnet: String,
}

#[derive(Serialize, Deserialize)]
pub struct Bulk {
    doc: Entry,
}
impl Bulk {
    pub fn new(magnet_details: TorrustTorrentStat) -> Bulk {
        let now = Utc::now();
        let checked_magnet = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let live_seeders = magnet_details.seeders as u64;
        let live_peers = magnet_details.leechers as u64;
        let live_grabs = magnet_details.completed as u64;

        Bulk {
            doc: Entry {
                live_seeders,
                live_peers,
                live_grabs,
                checked_magnet,
            },
        }
    }
}
