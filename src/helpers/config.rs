use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::path::Path;
extern crate sanitize_filename;

const CONFIGS_PATH: &str = "./channels";

fn deserialize_datetime_or_none<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper = Option::<String>::deserialize(deserializer);
    match helper {
        Ok(Some(date_str)) => match DateTime::parse_from_rfc3339(&date_str) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
            Err(_) => Ok(None),
        },
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub account: Option<Account>,
    #[serde(default)]
    pub channel: Option<Channel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    created_date: Option<DateTime<Utc>>,
    #[serde(default)]
    modified_date: Option<DateTime<Utc>>,
    #[serde(default)]
    joined_from: Option<String>,
    #[serde(default)]
    pub bible: Option<Bible>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_datetime_or_none")]
    pub join_date: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_datetime_or_none")]
    pub part_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub from_channel: Option<String>,
    #[serde(default)]
    pub bible: Option<Bible>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bible {
    #[serde(default)]
    pub last_translation: Option<String>,
    #[serde(default)]
    pub preferred_translation: Option<String>,
    #[serde(default)]
    pub last_verse: Option<String>,
    #[serde(default)]
    pub pending_text: Option<String>,
    #[serde(default)]
    pub votd: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    #[serde(default)]
    pub scriptures: Option<u32>,
    #[serde(default)]
    pub gospels_english: Option<u32>,
    #[serde(default)]
    pub gospels_spanish: Option<u32>,
    #[serde(default)]
    pub gospels_german: Option<u32>,
}

impl Config {
    fn default(username: &str) -> Self {
        let now = Utc::now();
        Config {
            account: Some(Account {
                username: Some(username.to_string()),
                notes: Some(String::new()),
                created_date: Some(now),
                modified_date: Some(now),
                joined_from: Some(String::new()),
                bible: Some(Bible {
                    last_translation: Some(String::new()),
                    last_verse: Some(String::new()),
                    pending_text: Some(String::new()),
                    votd: Some(String::new()),
                    preferred_translation: Some(String::new()),
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels_english: Some(0),
                    gospels_spanish: Some(0),
                    gospels_german: Some(0),
                }),
            }),
            channel: Some(Channel {
                notes: Some(String::new()),
                active: Some(false),
                join_date: None,
                part_date: None,
                from_channel: Some(String::new()),
                bible: Some(Bible {
                    last_translation: Some(String::new()),
                    last_verse: Some(String::new()),
                    pending_text: Some(String::new()),
                    votd: Some(String::new()),
                    preferred_translation: Some(String::new()),
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels_english: Some(0),
                    gospels_spanish: Some(0),
                    gospels_german: Some(0),
                }),
            }),
        }
    }

    pub fn load(username: &str) -> Self {
        let sanitized_username = sanitize_filename::sanitize(username);
        if fs::create_dir_all(CONFIGS_PATH).is_err() {
            return Self::default(&sanitized_username);
        }

        let file_path = format!("{}/{}.toml", CONFIGS_PATH, sanitized_username);
        let file_path = Path::new(&file_path);

        if !file_path.exists() {
            let new_config = Self::default(&sanitized_username);
            let _ = new_config.save();
            return new_config;
        }

        match fs::read_to_string(file_path) {
            Ok(content) => toml::from_str::<Self>(&content)
                .unwrap_or_else(|_| Self::default(&sanitized_username)),
            Err(_) => Self::default(&sanitized_username),
        }
    }

    pub fn save(&self) {
        let username = match self.account.as_ref().and_then(|a| a.username.as_ref()) {
            Some(username) => username,
            None => {
                eprintln!("Account or username is missing.");
                return;
            }
        };

        let sanitized_username = sanitize_filename::sanitize(username);
        let file_path = format!("{}/{}.toml", CONFIGS_PATH, sanitized_username);

        let toml_string = match toml::to_string(&self) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to serialize: {}", e);
                return;
            }
        };

        if let Err(e) = fs::write(file_path, toml_string) {
            eprintln!("Failed to write to file: {}", e);
        }
    }

    pub fn add_note(&mut self, note: String) {
        if let Some(account) = &mut self.account {
            let current_time = Utc::now();
            let current_notes = account.notes.take().unwrap_or_default();
            account.notes = Some(format!(
                "{} - {}\n{}",
                current_time.format("%Y-%m-%d %H:%M:%S UTC"),
                note,
                current_notes
            ));
            account.modified_date = Some(current_time);
            self.save();
        }
    }

    pub fn join_channel(&mut self, from_channel: String) {
        if let Some(channel) = &mut self.channel {
            channel.active = Some(true);
            channel.join_date = Some(Utc::now());
            channel.part_date = None;
            channel.from_channel = Some(from_channel);
            self.add_note("!joinchannel".to_owned());
            if let Some(account) = &mut self.account {
                account.modified_date = Some(Utc::now());
            }
            self.save()
        }
    }

    pub fn leave_channel(&mut self) {
        if let Some(channel) = &mut self.channel {
            channel.part_date = Some(Utc::now());
            channel.active = Some(false);
            self.add_note("!leavechannel".to_owned());
            if let Some(account) = &mut self.account {
                account.modified_date = Some(Utc::now());
            }
            self.save();
        }
    }

    pub fn set_last_verse(&mut self, verse: &str) {
        if let Some(account) = &mut self.account {
            if let Some(bible) = &mut account.bible {
                bible.last_verse = Some(verse.to_string());
                account.modified_date = Some(Utc::now());
                self.save();
            }
        }
    }

    pub fn get_last_verse_and_translation(&self) -> Option<(String, String)> {
        self.account.as_ref().and_then(|acc| {
            acc.bible.as_ref().and_then(|bible| {
                bible.last_verse.clone().and_then(|verse| {
                    bible
                        .last_translation
                        .clone()
                        .map(|translation| (verse, translation))
                })
            })
        })
    }

    pub fn get_translation(&self) -> Option<String> {
        self.account.as_ref().and_then(|acc| {
            acc.bible.as_ref().and_then(|bible| {
                bible
                    .preferred_translation
                    .clone()
                    .or_else(|| bible.last_translation.clone())
                    .filter(|s| !s.is_empty())
            })
        })
    }

    pub fn last_translation(&mut self, translation: &str) {
        if let Some(account) = &mut self.account {
            if let Some(bible) = &mut account.bible {
                bible.last_translation = Some(translation.to_string());
            }
            account.modified_date = Some(Utc::now());
        }
        self.save();
    }

    pub fn preferred_translation(&mut self, translation: &str) {
        if let Some(account) = &mut self.account {
            if let Some(bible) = &mut account.bible {
                bible.preferred_translation = Some(translation.to_string());
            }
            account.modified_date = Some(Utc::now());
        }
        self.add_note(format!("!translation {}", translation));
        self.save();
    }

    pub fn get_channels() -> Vec<String> {
        match fs::read_dir(CONFIGS_PATH) {
            Ok(entries) => entries
                .filter_map(|entry| {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(err) => {
                            println!("Failed to process an entry: {}", err);
                            return None;
                        }
                    };

                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(std::ffi::OsStr::to_str) == Some("toml")
                    {
                        match fs::read_to_string(&path) {
                            Ok(content) => match toml::from_str::<Config>(&content) {
                                Ok(config) => {
                                    if config.channel.as_ref()?.active.unwrap_or(false) {
                                        println!("Channel is active: {:?}", path);
                                        config.account?.username
                                    } else {
                                        None
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "Failed to deserialize TOML content from {}: {}",
                                        path.display(),
                                        err
                                    );
                                    None
                                }
                            },
                            Err(err) => {
                                println!("Failed to read file {}: {}", path.display(), err);
                                None
                            }
                        }
                    } else {
                        println!("Skipped non-TOML file or directory: {:?}", path);
                        None
                    }
                })
                .collect(),
            Err(err) => {
                println!("Failed to read directory '{}': {}", CONFIGS_PATH, err);
                Vec::new() // Return an empty vector if the directory read fails
            }
        }
    }

    pub fn from_channel(&self) -> String {
        self.channel
            .as_ref()
            .and_then(|c| c.from_channel.clone())
            .unwrap_or_default()
    }

    pub fn join_date(&self) -> String {
        self.channel
            .as_ref()
            .and_then(|channel| channel.join_date.as_ref())
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "No join date".to_string())
    }

    pub fn add_account_metrics_gospel_english(&mut self) {
        if let Some(account) = self.account.as_mut() {
            if let Some(metrics) = account.metrics.as_mut() {
                if let Some(gospels_english) = metrics.gospels_english.as_mut() {
                    *gospels_english += 1;
                } else {
                    metrics.gospels_english = Some(1);
                }
                self.save();
            }
        }
    }

    pub fn add_account_metrics_gospel_spanish(&mut self) {
        if let Some(account) = self.account.as_mut() {
            if let Some(metrics) = account.metrics.as_mut() {
                if let Some(gospels_spanish) = metrics.gospels_spanish.as_mut() {
                    *gospels_spanish += 1;
                } else {
                    metrics.gospels_spanish = Some(1);
                }
                self.save();
            }
        }
    }

    pub fn add_account_metrics_gospel_german(&mut self) {
        if let Some(account) = self.account.as_mut() {
            if let Some(metrics) = account.metrics.as_mut() {
                if let Some(gospels_german) = metrics.gospels_german.as_mut() {
                    *gospels_german += 1;
                } else {
                    metrics.gospels_german = Some(1);
                }
                self.save();
            }
        }
    }

    pub fn add_account_metrics_scriptures(&mut self) {
        let success = self
            .account
            .as_mut()
            .and_then(|acc| acc.metrics.as_mut())
            .and_then(|mtr| mtr.scriptures.as_mut())
            .map(|scriptures| {
                *scriptures += 1;
            })
            .is_some();
        if success {
            self.save();
        }
    }
}
