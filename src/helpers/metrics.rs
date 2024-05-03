use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};
use std::{fs, io::Result};

const METRICS_PATH: &str = "./metrics";

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    #[serde(default)]
    pub channels: Option<u32>,
    #[serde(default)]
    pub users: Option<u32>,
    #[serde(default)]
    pub scriptures: Option<u32>,
    #[serde(default)]
    pub gospels_english: Option<u32>,
    #[serde(default)]
    pub gospels_spanish: Option<u32>,
    #[serde(default)]
    pub gospels_german: Option<u32>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            channels: Some(0),
            users: Some(0),
            scriptures: Some(0),
            gospels_english: Some(0),
            gospels_spanish: Some(0),
            gospels_german: Some(0),
        }
    }
}

impl Metrics {
    pub fn load_metrics() -> Result<()> {
        let metrics_dir = Path::new(METRICS_PATH);
        if !metrics_dir.exists() {
            fs::create_dir_all(metrics_dir)?;
        }

        let latest_file = fs::read_dir(metrics_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "toml")
            })
            .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap());

        let metrics: Metrics = match latest_file {
            Some(file) => {
                let contents = fs::read_to_string(file.path())?;
                toml::from_str(&contents)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            }
            None => Metrics::default(),
        };

        let file_name = format!("metrics_{date}.toml", date = Local::now().format("%Y%m%d"));
        let file_path = metrics_dir.join(file_name);
        let toml_string = toml::to_string(&metrics)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(file_path, toml_string)?;

        Ok(())
    }
}
