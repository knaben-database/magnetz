use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TorrustTorrentStat {
    seeders: u32,
    completed: u32,
    leechers: u32,
}
impl fmt::Display for TorrustTorrentStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{seeders: {}, completed: {}, leechers: {}}}",
            self.seeders, self.completed, self.leechers
        )
    }
}

#[derive(Serialize, Deserialize)]
struct TorrustScrapeDetails {
    transaction_id: i32,
    torrent_stats: Vec<TorrustTorrentStat>,
}

#[derive(Serialize, Deserialize)]
pub struct TorrustScraper {
    #[serde(rename = "Scrape")]
    scrape: TorrustScrapeDetails,
}
pub type Scraper = HashMap<String, TorrustTorrentStat>;
impl TorrustScraper {
    pub fn merge_ids(self, ids: Vec<String>) -> Scraper {
        ids.into_iter()
            .zip(self.scrape.torrent_stats.into_iter())
            .collect()
    }
}
impl fmt::Display for TorrustScraper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction ID {}\n  {}",
            self.scrape.transaction_id,
            self.scrape
                .torrent_stats
                .iter()
                .map(|ts| ts.to_string())
                .collect::<Vec<String>>()
                .join("\n  ")
        )
    }
}
