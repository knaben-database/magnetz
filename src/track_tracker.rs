use std::collections::HashMap;
use std::process::Command;

use super::magnet::Magnet;
use super::scraper::Scraper;
use super::tracker::Tracker;
use super::tracker::TrackerSchema;

#[derive(Debug)]
pub struct TrackTrackers {
    tracker: Tracker,
    magnets: HashMap<String, Magnet>,
    score: f32,
    score_multiplier: f32,
}
impl TrackTrackers {
    pub fn new(tracker: Tracker) -> TrackTrackers {
        TrackTrackers {
            tracker,
            magnets: HashMap::new(),
            score: 0.00,
            score_multiplier: 0.00,
        }
    }

    pub fn insert_magnet(&mut self, (id, magnet): (String, Magnet)) {
        self.magnets.insert(id, magnet);
    }

    pub fn scrape_all(&mut self) {
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

        println!(
            "Trying command: {} {} {} {}",
            format!("torrust-tracker/target/release/{}", application),
            "scrape",
            self.tracker.to_string(),
            arguments.first().unwrap()
        );

        if let Err(e) = command_result {
            println!(
                "Scrape all for {} failed with this error: {}",
                self.tracker, e
            );
            self.change_score(-3.0);
            return;
        }

        let command_result = command_result.unwrap();

        if !command_result.status.success() {
            print!("Failed: {}", self.tracker);
            if !command_result.stderr.is_empty() {
                print!(
                    " because of: {}",
                    String::from_utf8_lossy(&command_result.stderr)
                )
            }
            println!("");
            self.change_score(-1.5);
            return;
        }

        let output_str = String::from_utf8_lossy(&command_result.stdout);

        if let Some(index) = output_str.find("{") {
            let json_str = &output_str[index..];
            println!("{}", json_str);
            self.change_score(1.0);
        } else {
            println!("Pattern not found.");
            self.change_score(-2.0);
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
            self.score_multiplier += 1.0 * weight
        }

        self.score = (self.score + self.score_multiplier * 0.1).clamp(0.0, 1.0);
        println!("{} score is now {}", self.tracker, self.score);
    }
}
