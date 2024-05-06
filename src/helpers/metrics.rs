use chrono::Local;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::Path;
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
        match Metrics::load_metrics() {
            Ok(metrics) => metrics,
            Err(_) => Metrics {
                channels: Some(0),
                users: Some(0),
                scriptures: Some(0),
                gospels_english: Some(0),
                gospels_spanish: Some(0),
                gospels_german: Some(0),
            },
        }
    }
}
impl Metrics {
    fn load_metrics() -> Result<Self> {
        let metrics_dir = Path::new(METRICS_PATH);
        if !metrics_dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No metrics file found",
            ));
        }

        let latest_file = fs::read_dir(metrics_dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "toml")
            })
            .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap());

        match latest_file {
            Some(file) => {
                let contents = fs::read_to_string(file.path())?;
                toml::from_str(&contents)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No metrics file found",
            )),
        }
    }

    fn save(&self) {
        let metrics_dir = Path::new(METRICS_PATH);
        if !metrics_dir.exists() {
            let _ = fs::create_dir_all(metrics_dir);
        }

        let file_name = format!("metrics_{}.toml", Local::now().format("%Y%m%d"));
        let file_path = metrics_dir.join(file_name);
        let toml_string = toml::to_string(self).unwrap();

        let _ = fs::write(file_path, toml_string);
    }

    pub fn increment_channels(&mut self) {
        if let Some(channels) = self.channels {
            self.channels = Some(channels + 1);
        }
        self.save();
    }

    pub fn reset_channels(&mut self) {
        if let Some(channels) = self.channels {
            self.channels = Some(0);
        }
        self.save();
    }

    pub fn increment_users(&mut self) {
        if let Some(users) = self.users {
            self.users = Some(users + 1);
        }
        self.save();
    }

    pub fn reset_users(&mut self) {
        if let Some(channels) = self.channels {
            self.users = Some(0);
        }
        self.save();
    }

    pub fn increment_scriptures(&mut self) {
        if let Some(scriptures) = self.scriptures {
            self.scriptures = Some(scriptures + 1);
        }
        self.save();
    }

    pub fn increment_gospels_english(&mut self) {
        if let Some(gospels_english) = self.gospels_english {
            self.gospels_english = Some(gospels_english + 1);
        }
        self.save();
    }

    pub fn increment_gospels_spanish(&mut self) {
        if let Some(gospels_spanish) = self.gospels_spanish {
            self.gospels_spanish = Some(gospels_spanish + 1);
        }
        self.save();
    }

    pub fn increment_gospels_german(&mut self) {
        if let Some(gospels_german) = self.gospels_german {
            self.gospels_german = Some(gospels_german + 1);
        }
        self.save();
    }
}
