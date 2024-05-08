use chrono::Utc;
use regex::Regex;
use reqwest::{self, StatusCode};
use std::error::Error;

use super::ConfigManager;

pub async fn fetch_verse_of_the_day() -> Result<String, Box<dyn Error>> {
    let today = Utc::now().format("%Y%m%d").to_string();
    let mut config_manager = ConfigManager::new("./votd_config.toml");
    let mut votd = config_manager.get_string(&today, "");

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
            config_manager.set_string(&today, &votd);
        } else {
            return Err("Verse reference not found".into());
        }
    }

    Ok(votd)
}
