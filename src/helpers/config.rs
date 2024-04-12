use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, read_dir};
use std::path::Path;

const CONFIGS_PATH: &str = "./channels";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub account: Option<Account>,
    pub channel: Option<Channel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub username: Option<String>,
    pub notes: Option<String>,
    date_created: Option<String>,
    date_modified: Option<String>,
    joined_from: Option<String>,
    pub bible: Option<Bible>,
    pub metrics: Option<Metrics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub notes: Option<String>,
    broadcaster: Option<bool>,
    date_joined: Option<String>,
    pub bible: Option<Bible>,
    pub metrics: Option<Metrics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bible {
    pub translation: Option<String>,
    pub last_verse: Option<String>,
    pub votd: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub scriptures: Option<u32>,
    pub gospels: Option<u32>,
}

impl Config {
    fn default(username: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Config {
            account: Some(Account {
                username: Some(username.to_string()),
                notes: Some(String::new()),
                date_created: Some(now.clone()),
                date_modified: Some(now.clone()),
                joined_from: Some(String::new()),
                bible: Some(Bible {
                    translation: Some(String::new()),
                    last_verse: Some(String::new()),
                    votd: Some(String::new()),
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels: Some(0),
                }),
            }),
            channel: Some(Channel {
                notes: Some(String::new()),
                broadcaster: Some(false),
                date_joined: Some(String::new()),
                bible: Some(Bible {
                    translation: Some(String::new()),
                    last_verse: Some(String::new()),
                    votd: Some(String::new()),
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels: Some(0),
                }),
            }),
        }
    }

    pub fn load(username: &str) -> Self {
        if fs::create_dir_all(CONFIGS_PATH).is_err() {
            return Self::default(username);
        }

        let file_path = format!("{}/{}.toml", CONFIGS_PATH, username);
        let file_path = Path::new(&file_path);

        if !file_path.exists() {
            return Self::default(username);
        }

        match fs::read_to_string(file_path) {
            Ok(content) => {
                toml::from_str::<Self>(&content).unwrap_or_else(|_| Self::default(username))
            }
            Err(_) => Self::default(username),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!(
            "./channels/{}.toml",
            self.account.as_ref().unwrap().username.as_ref().unwrap()
        );
        let toml_string = toml::to_string(&self)?;
        fs::write(file_path, toml_string)?;
        Ok(())
    }

    pub fn add_note(&mut self, note: String) {
        if let Some(account) = &mut self.account {
            let current_time = Utc::now().to_rfc3339();
            let current_notes = account.notes.take().unwrap_or_default();
            account.notes = Some(format!("{} - {}\n{}", current_time, note, current_notes));
            account.date_modified = Some(current_time);
            self.save()
                .unwrap_or_else(|e| eprintln!("Failed to save: {}", e));
        }
    }

    pub fn join_channel(&mut self) {
        if let Some(channel) = &mut self.channel {
            channel.broadcaster = Some(true);
            channel.date_joined = Some(Utc::now().to_rfc3339());
            self.add_note("!joinchannel".to_owned());
            if let Some(account) = &mut self.account {
                account.date_modified = Some(Utc::now().to_rfc3339());
            }
            self.save()
                .unwrap_or_else(|e| eprintln!("Failed to save: {}", e));
        }
    }

    pub fn get_channels() -> Vec<String> {
        read_dir(CONFIGS_PATH)
            .unwrap_or_else(|_| panic!("Failed to read directory"))
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_file()
                        && path.extension().and_then(std::ffi::OsStr::to_str) == Some("toml")
                    {
                        fs::read_to_string(path).ok().and_then(|content| {
                            toml::from_str::<Config>(&content)
                                .ok()
                                .and_then(|config| config.account?.username)
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}
