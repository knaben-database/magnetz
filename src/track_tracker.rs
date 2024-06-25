use anyhow::{bail, Result};
use rand::Rng;
use std::collections::HashMap;
use std::process::Command;

use super::magnet::Magnet;
use super::scraper::Scraper;
use super::scraper::TorrustScraper;
use super::tracker::Tracker;
use super::tracker::TrackerSchema;

const TIMEOUT_COMMAND: u32 = 5; // seconds
const MAX_MULTIPLIER_DEVIATION: f32 = 50.0;
const TRACKER_DEAD_HOPE_PERCENTAGE: f32 = 0.005; // 0.5% chance to check a dead tracker
const TRACKER_DOUBT_PERCENTAGE: f32 = 0.5; // If the score is lower than this, we doubt that it is online

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
            score: TRACKER_DOUBT_PERCENTAGE, // Percentage number between 0 and 1
            score_multiplier: 0.00,          // Exponential multiplier between -50 and 50
        }
    }

    pub fn insert_magnet(&mut self, (id, magnet): (String, Magnet)) {
        self.magnets.insert(id, magnet);
    }

    pub fn clear_magnets(&mut self) {
        self.magnets.clear();
    }

    pub fn scrape_all(&mut self) -> Result<Scraper> {
        // No need to reach out to the tracker if it has no magnets
        if self.magnets.is_empty() {
            return Ok(Scraper::new());
        }

        // if score is 0, there is a 99.5% chance we will skip checking it
        let num: f32 = rand::thread_rng().gen_range(0.0..1.0);
        if TRACKER_DOUBT_PERCENTAGE > self.score && num > self.score + TRACKER_DEAD_HOPE_PERCENTAGE
        {
            bail!("Tracker {} is likely down, skipping", self.tracker);
        } else if TRACKER_DOUBT_PERCENTAGE > self.score {
            println!(
                "Testing tracker, {} > {}",
                self.score + TRACKER_DEAD_HOPE_PERCENTAGE,
                num
            );
        }

        let application = match self.tracker.schema {
            TrackerSchema::UDP => "udp_tracker_client",
            TrackerSchema::HTTP | TrackerSchema::HTTPS => "http_tracker_client",
        };
        let mut arguments: Vec<String> = vec!["scrape".to_string(), self.tracker.to_string()];
        arguments.extend(
            self.magnets
                .values()
                .map(|m| m.clone().info_hash)
                .collect::<Vec<String>>(),
        );

        let output_command = run_command(
            format!("torrust-tracker/target/release/{}", application),
            arguments,
        );

        let output_str = match output_command {
            Ok(r) => r,
            Err(e) => {
                self.change_score(-1.5);
                bail!(e);
            }
        };

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
            self.score_multiplier = (self.score_multiplier + 1.0 * weight)
                .clamp(-MAX_MULTIPLIER_DEVIATION, MAX_MULTIPLIER_DEVIATION);
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

fn run_command(application: String, arguments: Vec<String>) -> Result<String> {
    let command_result = Command::new(application).args(arguments.clone()).output()?;

    if !command_result.status.success() {
        let status_code = command_result
            .status
            .code()
            .and_then(|n| Some(n.to_string()))
            .unwrap_or("?".to_string());
        let mut fail_text = format!("Failed status code {}", status_code);
        if !command_result.stderr.is_empty() {
            fail_text += &format!(
                " because of: {}",
                String::from_utf8_lossy(&command_result.stderr).trim()
            );
        }
        bail!(fail_text);
    }

    Ok(String::from_utf8_lossy(&command_result.stdout).into_owned())
}
