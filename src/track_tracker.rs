use anyhow::{bail, Result};
use rand::Rng;
use std::collections::HashMap;
use std::process::Command;

use super::magnet::Magnet;
use super::scraper::Scraper;
use super::scraper::TorrustScraper;
use super::tracker::Tracker;
use super::tracker::TrackerSchema;

#[derive(Debug)]
pub struct TrackTrackers {
    pub tracker: Tracker,
    magnets: HashMap<String, Magnet>,
    score: f32,
    score_multiplier: f32,
}
impl TrackTrackers {
    pub fn new(tracker: Tracker) -> TrackTrackers {
        TrackTrackers {
            tracker,
            magnets: HashMap::new(),
            score: 0.50,            // Percentage number between 0 and 1
            score_multiplier: 0.00, // Exponential multiplier between -50 and 50
        }
    }

    pub fn insert_magnet(&mut self, (id, magnet): (String, Magnet)) {
        self.magnets.insert(id, magnet);
    }

    pub fn clear_magnets(&mut self) {
        self.magnets.clear();
    }

    pub fn scrape_all(&mut self) -> Result<Scraper> {
        // if score is 0, there is a 99.5% chance we will skip checking it
        let num: f32 = rand::thread_rng().gen_range(0.0..1.0);
        if 0.5 > self.score && num > self.score + 0.005 {
            bail!("Tracker {} is likely down, skipping", self.tracker);
        } else if 0.5 > self.score {
            println!("Testing tracker, {} > {}", self.score + 0.005, num);
        }

        let arguments: Vec<String> = self.magnets.values().map(|m| m.clone().info_hash).collect();
        let application = match self.tracker.schema {
            TrackerSchema::UDP => "udp_tracker_client",
            TrackerSchema::HTTP | TrackerSchema::HTTPS => "http_tracker_client",
        };
        let command_result =
            Command::new(format!("torrust-tracker/target/release/{}", application))
                .arg("scrape")
                .arg(self.tracker.to_string())
                .args(arguments.clone())
                .output();

        if let Err(e) = command_result {
            self.change_score(-3.0);
            bail!(e);
        }

        let command_result = command_result.unwrap();

        if !command_result.status.success() {
            let mut fail_text = format!(
                "{}: Failed status code {}",
                self.tracker,
                command_result
                    .status
                    .code()
                    .and_then(|n| Some(n.to_string()))
                    .unwrap_or("?".to_string())
            );
            if !command_result.stderr.is_empty() {
                fail_text += &format!(
                    " because of: {}",
                    String::from_utf8_lossy(&command_result.stderr).trim()
                );
            }
            self.change_score(-1.5);
            bail!(fail_text);
        }

        let output_str = String::from_utf8_lossy(&command_result.stdout);

        if let Some(index) = output_str.find("{") {
            let json_str = &output_str[index..];
            let json: TorrustScraper = serde_json::from_str(json_str).map_err(|e| {
                self.change_score(-0.5);
                e
            })?;
            let merged_json: Scraper = json.merge_ids(self.magnets.clone().into_keys().collect());
            self.change_score(1.0);
            return Ok(merged_json);
        } else {
            self.change_score(-2.0);
            bail!("JSON pattern not found in response")
        }
    }

    fn change_score(&mut self, weight: f32) {
        if 0.0 > weight && self.score_multiplier > 0.0 {
            // Switch from positive to minus
            self.score_multiplier = -1.0;
        } else if weight > 0.0 && 0.0 > self.score_multiplier {
            // Switch from minus to positive
            self.score_multiplier = 1.0;
        } else {
            self.score_multiplier = (self.score_multiplier + 1.0 * weight).clamp(-50.0, 50.0);
        }

        let new_score = (self.score + self.score_multiplier * 0.05).clamp(0.0, 1.0);
        if new_score != self.score {
            self.score = new_score;
            println!(
                "{}: score is now {} ({})",
                self.tracker,
                self.score,
                self.score_multiplier * 0.05
            );
        }
    }
}
