use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;
use std::{fs, io::Result};

const METRICS_PATH: &str = "./metrics";

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub name: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    #[serde(default)]
    pub daily_channel_counts: HashMap<String, u32>,
    #[serde(default)]
    pub channels: Option<u32>,
    #[serde(default)]
    pub channels_list: Vec<Channel>,
    #[serde(default)]
    pub users: Option<u32>,
    #[serde(default)]
    pub users_list: Vec<User>,
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
                channels_list: vec![],
                users_list: vec![],
                scriptures: Some(0),
                gospels_english: Some(0),
                gospels_spanish: Some(0),
                gospels_german: Some(0),
                daily_channel_counts: HashMap::new(),
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
                let mut metrics: Self = toml::from_str(&contents)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                //The channels are loaded from the file system each time the application is started, so these metrics can be cleared out.
                metrics.channels_list.clear();
                metrics.channels = Some(0);
                let date_key = Local::now().format("%Y%m%d").to_string();
                metrics.daily_channel_counts.remove_entry(&date_key);
                Ok(metrics)
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
        //let file_name = format!("metrics_{}.toml", Local::now().format("%Y%m%d%H%M%S"));
        let file_path = metrics_dir.join(file_name);
        let toml_string = toml::to_string(self).unwrap();

        let _ = fs::write(file_path, toml_string);
    }

    pub fn add_channel(&mut self, name: &str) {
        if !self.channels_list.iter().any(|user| user.name == name) {
            self.channels_list.push(Channel {
                name: name.to_string(),
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
            self.channels = Some(self.channels_list.len().try_into().unwrap());
            let date_key = Local::now().format("%Y%m%d").to_string();
            let count = self.daily_channel_counts.entry(date_key).or_insert(0);
            *count += 1;
            self.save();
        }
    }

    pub fn remove_channel(&mut self, name: &str) {
        if let Some(pos) = self
            .channels_list
            .iter()
            .position(|channel| channel.name == name)
        {
            self.channels_list.remove(pos);
            self.channels = Some(self.channels_list.len().try_into().unwrap());
            self.save();
        }
    }

    pub fn add_user(&mut self, name: &str) {
        if !self.users_list.iter().any(|user| user.name == name) {
            self.users_list.push(User {
                name: name.to_string(),
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
            self.users = Some(self.users_list.len().try_into().unwrap());
            self.save();
        }
    }

    pub fn increment_scriptures(&mut self) {
        self.scriptures = Some(self.scriptures.unwrap_or_default() + 1);
        self.save();
    }

    pub fn increment_gospels_english(&mut self) {
        self.gospels_english = Some(self.gospels_english.unwrap_or_default() + 1);
        self.save();
    }

    pub fn increment_gospels_spanish(&mut self) {
        self.gospels_spanish = Some(self.gospels_spanish.unwrap_or_default() + 1);
        self.save();
    }

    pub fn increment_gospels_german(&mut self) {
        self.gospels_german = Some(self.gospels_german.unwrap_or_default() + 1);
        self.save();
    }
}
