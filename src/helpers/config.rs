use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub account: Account,
    pub channel: Channel,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub username: String,
    pub notes: String,
    date_created: String,
    date_modified: String,
    joined_from: String,
    pub bible: Bible,
    pub metrics: Metrics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub notes: String,
    broadcaster: bool,
    pub bible: Bible,
    pub metrics: Metrics,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Bible {
    pub translation: String,
    pub last_verse: String,
    pub votd: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub scriptures: u32,
    pub gospels: u32,
}
impl Config {
    pub fn load(username: &str) -> Self {
        let path = "./channels";
        if fs::create_dir_all(path).is_err() {
            return Self::default(username);
        }

        let file_path = format!("{}/{}.toml", path, username);
        let file_path = Path::new(&file_path);

        if !file_path.exists() {
            return Self::default(username);
        }

        let content = fs::read_to_string(file_path);
        match content {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(config) => config,
                Err(_) => Self::default(username),
            },
            Err(_) => Self::default(username),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("./channels/{}.toml", self.account.username);
        let toml_string = toml::to_string(&self)?;
        fs::write(file_path, toml_string)?;
        Ok(())
    }

    fn default(username: &str) -> Self {
        // Get current time
        let now = Utc::now();
        Config {
            account: Account {
                username: username.to_string(),
                notes: String::new(),
                date_created: now.to_rfc3339(),
                date_modified: now.to_rfc3339(),
                joined_from: String::new(),
                bible: Bible {
                    translation: String::new(),
                    last_verse: String::new(),
                    votd: String::new(),
                },
                metrics: Metrics {
                    scriptures: 0,
                    gospels: 0,
                },
            },
            channel: Channel {
                notes: String::new(),
                broadcaster: false,
                bible: Bible {
                    translation: String::new(),
                    last_verse: String::new(),
                    votd: String::new(),
                },
                metrics: Metrics {
                    scriptures: 0,
                    gospels: 0,
                },
            },
        }
    }
    pub fn add_note(&mut self, note: String) {
        self.account.notes = format!(
            "{} - {}\n{}",
            Utc::now().to_rfc3339(),
            note,
            self.account.notes
        );
        self.account.date_modified = Utc::now().to_rfc3339();
        if let Err(e) = self.save() {
            eprintln!("Failed to save: {}", e);
        }
    }
    pub fn set_broadcaster(&mut self, broadcaster: bool) {
        self.channel.broadcaster = broadcaster;
        self.account.date_modified = Utc::now().to_rfc3339();
        if let Err(e) = self.save() {
            eprintln!("Failed to save: {}", e);
        }
    }
}
