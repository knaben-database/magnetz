use std::env;

fn default_config() -> Config {
    Config {
        elasticsearch_host: "127.0.0.1".to_string(),
        elasticsearch_port: 9200,
        elasticsearch_index: "torrent_cache2".to_string(),
        redis_host: "127.0.0.1".to_string(),
        redis_port: 6379,
        redis_key: "heatmap".to_string(),
        torrust_path: "torrust-tracker/target/release".to_string(),
        lock_timer: 7200,
    }
}

pub struct Config {
    pub elasticsearch_host: String,
    pub elasticsearch_port: u16,
    pub elasticsearch_index: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_key: String,
    pub torrust_path: String,
    pub lock_timer: u32,
}
impl Config {
    pub fn new() -> Self {
        let default_config = default_config();

        let elasticsearch_host: String =
            env::var("ELASTICSEARCH_HOST").unwrap_or(default_config.elasticsearch_host);
        let elasticsearch_port: u16 = env::var("ELASTICSEARCH_PORT")
            .unwrap_or(default_config.elasticsearch_port.to_string())
            .parse()
            .unwrap_or(default_config.elasticsearch_port);
        let elasticsearch_index: String =
            env::var("ELASTICSEARCH_INDEX").unwrap_or(default_config.elasticsearch_index);
        let redis_host: String = env::var("REDIS_HOST").unwrap_or(default_config.redis_host);
        let redis_port: u16 = env::var("REDIS_PORT")
            .unwrap_or(default_config.redis_port.to_string())
            .parse()
            .unwrap_or(default_config.redis_port);
        let redis_key: String = env::var("REDIS_KEY").unwrap_or(default_config.redis_key);
        let torrust_path: String = env::var("TORRUST_PATH").unwrap_or(default_config.torrust_path);
        let lock_timer: u32 = env::var("LOCK_TIMER")
            .unwrap_or(default_config.lock_timer.to_string())
            .parse()
            .unwrap_or(default_config.lock_timer);

        Self {
            elasticsearch_host,
            elasticsearch_port,
            elasticsearch_index,
            redis_host,
            redis_port,
            redis_key,
            torrust_path,
            lock_timer,
        }
    }
}
