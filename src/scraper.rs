struct TorrentStat {
    seeders: u32,
    completed: u32,
    leechers: u32,
}

pub struct Scraper {
    transaction_id: i32,
    torrent_stats: Vec<TorrentStat>,
}
