use chrono::{NaiveTime, Utc};
use regex::Regex;
use reqwest::{self, StatusCode};
use std::error::Error;

use super::ConfigManager;

pub async fn fetch_verse_of_the_day() -> Result<String, Box<dyn Error>> {
    let mut config_manager = ConfigManager::new("./votd_config.toml");
    let switch_time_str = config_manager.get_string("NEW_DAY_TIME", "01:00");
    let switch_time = NaiveTime::parse_from_str(&switch_time_str, "%H:%M").unwrap_or_default();
    let current_datetime = Utc::now();

    let date_to_check = if current_datetime.time() > switch_time {
        current_datetime.format("%Y%m%d").to_string()
    } else {
        (current_datetime - chrono::Duration::days(1))
            .format("%Y%m%d")
            .to_string()
    };

    // println!("{}:", date_to_check);
    let mut votd = config_manager.get_string(&date_to_check, "");

    if votd.is_empty() {
        let url = config_manager.get_string("VOTD_URL", "https://bible-api.com/?random=verse");
        let regex_pattern = config_manager.get_string("VOTD_REGEX", r#""reference":"([^"]+)"#);
        let response = reqwest::Client::new().get(&url).send().await?;

        if response.status() != StatusCode::OK {
            return Err(format!("Failed to fetch data: {}", response.status()).into());
        }

        let body = response.text().await?;
        let re = Regex::new(&regex_pattern)?;

        if let Some(caps) = re.captures(&body) {
            votd = caps.get(1).unwrap().as_str().to_string();
            config_manager.set_string(&date_to_check, &votd);
        } else {
            return Err("Verse reference not found".into());
        }
    }

    Ok(votd)
}
