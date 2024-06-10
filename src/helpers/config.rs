use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::path::Path;
use log::info;

extern crate sanitize_filename;

const CONFIGS_PATH: &str = "./channels";

fn deserialize_datetime_or_none<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper = Option::<String>::deserialize(deserializer)?;
    if let Some(date_str) = helper {
        match DateTime::parse_from_rfc3339(&date_str) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
            Err(_) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn serialize_optional_string<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(v),
        None => serializer.serialize_none(),
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
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub username: Option<String>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub notes: Option<String>,
    #[serde(default)]
    pub created_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub modified_date: Option<DateTime<Utc>>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub joined_from: Option<String>,
    #[serde(default)]
    pub bible: Option<Bible>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub notes: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_datetime_or_none")]
    pub join_date: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_datetime_or_none")]
    pub part_date: Option<DateTime<Utc>>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub from_channel: Option<String>,
    #[serde(default)]
    pub bible: Option<Bible>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
    #[serde(default = "default_command_prefix")]
    pub command_prefix: Option<char>,
    #[serde(default)]
    pub modified_date: Option<DateTime<Utc>>,
}

fn default_command_prefix() -> Option<char> {
    Some('!')
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bible {
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub last_translation: Option<String>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub preferred_translation: Option<String>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub last_verse: Option<String>,
    #[serde(default, serialize_with = "serialize_optional_string")]
    pub pending_text: Option<String>,
    #[serde(default, serialize_with = "serialize_optional_string")]
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

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            scriptures: Some(0),
            gospels_english: Some(0),
            gospels_spanish: Some(0),
            gospels_german: Some(0),
        }
    }
}

impl Config {
    fn default(username: &str) -> Self {
        let now = Utc::now();
        Config {
            account: Some(Account {
                username: Some(username.to_string()),
                notes: None,
                created_date: Some(now),
                modified_date: Some(now),
                joined_from: None,
                bible: Some(Bible {
                    last_translation: None,
                    last_verse: None,
                    pending_text: None,
                    votd: None,
                    preferred_translation: None,
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels_english: Some(0),
                    gospels_spanish: Some(0),
                    gospels_german: Some(0),
                }),
            }),
            channel: Some(Channel {
                notes: None,
                active: Some(false),
                join_date: None,
                part_date: None,
                from_channel: None,
                bible: Some(Bible {
                    last_translation: None,
                    last_verse: None,
                    pending_text: None,
                    votd: None,
                    preferred_translation: None,
                }),
                metrics: Some(Metrics {
                    scriptures: Some(0),
                    gospels_english: Some(0),
                    gospels_spanish: Some(0),
                    gospels_german: Some(0),
                }),
                command_prefix: Some('!'),
                modified_date: Some(now),
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
        if let Some(account) = self.account.as_mut() {
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

    pub fn join_channel(&mut self, from_channel: &str) {
        if let Some(channel) = self.channel.as_mut() {
            channel.active = Some(true);
            channel.join_date = Some(Utc::now());
            channel.part_date = None;
            channel.from_channel = Some(from_channel.to_owned());
            channel.modified_date = Some(Utc::now());
            self.add_note("!joinchannel".to_owned());
            if let Some(account) = self.account.as_mut() {
                account.modified_date = Some(Utc::now());
            }
            self.save()
        }
    }

    pub fn leave_channel(&mut self) {
        if let Some(channel) = self.channel.as_mut() {
            channel.part_date = Some(Utc::now());
            channel.active = Some(false);
            channel.modified_date = Some(Utc::now());
            self.add_note("!leavechannel".to_owned());
            if let Some(account) = self.account.as_mut() {
                account.modified_date = Some(Utc::now());
            }
            self.save();
        }
    }

    pub fn set_last_verse(&mut self, verse: &str) {
        if let Some(account) = self.account.as_mut() {
            if let Some(bible) = account.bible.as_mut() {
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
        if let Some(account) = self.account.as_mut() {
            if let Some(bible) = account.bible.as_mut() {
                bible.last_translation = Some(translation.to_string());
            }
            account.modified_date = Some(Utc::now());
        }
        self.save();
    }

    pub fn preferred_translation(&mut self, translation: &str) {
        if let Some(account) = self.account.as_mut() {
            if let Some(bible) = account.bible.as_mut() {
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
                            eprintln!("Failed to process an entry: {}", err);
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
                                        info!(
                                            "Channel is active: {}",
                                            config.account.as_ref()?.username.as_ref()?.to_string()
                                        );
                                        config.account?.username
                                    } else {
                                        None
                                    }
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Failed to deserialize TOML content from {}: {}",
                                        path.display(),
                                        err
                                    );
                                    None
                                }
                            },
                            Err(err) => {
                                eprintln!("Failed to read file {}: {}", path.display(), err);
                                None
                            }
                        }
                    } else {
                        // eprintln!("Skipped non-TOML file or directory: {:?}", path);
                        None
                    }
                })
                .collect(),
            Err(err) => {
                eprintln!("Failed to read directory '{}': {}", CONFIGS_PATH, err);
                Vec::new() // Return an empty vector if the directory read fails
            }
        }
    }

    pub fn delete(&self) {
        if let Some(username) = self.account.as_ref().and_then(|a| a.username.as_ref()) {
            let sanitized_username = sanitize_filename::sanitize(username);
            let file_path = format!("{}/{}.toml", CONFIGS_PATH, sanitized_username);
            let path = Path::new(&file_path);
            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Failed to delete file {}: {}", file_path, e);
                }
            }
        } else {
            eprintln!("Username is missing, cannot delete file.");
        }
    }

    pub fn get_from_channel(&self) -> String {
        self.channel
            .as_ref()
            .and_then(|c| c.from_channel.clone())
            .unwrap_or_default()
    }

    pub fn get_command_prefix(&self) -> char {
        self.channel
            .as_ref()
            .and_then(|c| c.command_prefix.clone())
            .unwrap_or_default()
    }

    pub fn set_command_prefix(&mut self, prefix: &char) {
        if let Some(channel) = self.channel.as_mut() {
            channel.command_prefix = Some(*prefix);
            channel.modified_date = Some(Utc::now());
            self.save();
        }
    }

    pub fn get_votd(&self) -> Option<String> {
        self.channel
            .as_ref()
            .and_then(|c| c.bible.as_ref().and_then(|b| b.votd.clone()))
    }

    pub fn set_votd(&mut self, reference: Option<String>) {
        if let Some(channel) = self.channel.as_mut() {
            if let Some(bible) = channel.bible.as_mut() {
                bible.votd = reference;
                channel.modified_date = Some(Utc::now());
                self.save();
            }
        }
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
        if let Some(gospels_english) = self
            .account
            .as_mut()
            .and_then(|acc| acc.metrics.as_mut())
            .and_then(|mtr| mtr.gospels_english.as_mut())
        {
            *gospels_english += 1;
        } else {
            if let Some(metrics) = self.account.as_mut().and_then(|acc| acc.metrics.as_mut()) {
                metrics.gospels_english = Some(1);
            }
        }
        self.save();
    }

    pub fn add_account_metrics_gospel_spanish(&mut self) {
        if let Some(gospels_spanish) = self
            .account
            .as_mut()
            .and_then(|acc| acc.metrics.as_mut())
            .and_then(|mtr| mtr.gospels_spanish.as_mut())
        {
            *gospels_spanish += 1;
        } else {
            if let Some(metrics) = self.account.as_mut().and_then(|acc| acc.metrics.as_mut()) {
                metrics.gospels_spanish = Some(1);
            }
        }
        self.save();
    }

    pub fn add_account_metrics_gospel_german(&mut self) {
        if let Some(gospels_german) = self
            .account
            .as_mut()
            .and_then(|acc| acc.metrics.as_mut())
            .and_then(|mtr| mtr.gospels_german.as_mut())
        {
            *gospels_german += 1;
        } else {
            if let Some(metrics) = self.account.as_mut().and_then(|acc| acc.metrics.as_mut()) {
                metrics.gospels_german = Some(1);
            }
        }
        self.save();
    }

    pub fn add_account_metrics_scriptures(&mut self) {
        if let Some(scriptures) = self
            .account
            .as_mut()
            .and_then(|acc| acc.metrics.as_mut())
            .and_then(|mtr| mtr.scriptures.as_mut())
        {
            *scriptures += 1;
            self.save();
        }
    }

    pub fn add_channel_metrics_gospel_english(&mut self) {
        if let Some(channel) = self.channel.as_mut() {
            let metrics = channel.metrics.get_or_insert_with(Default::default);
            metrics.gospels_english = Some(metrics.gospels_english.unwrap_or(0) + 1);
            self.save();
        }
    }

    pub fn add_channel_metrics_gospel_spanish(&mut self) {
        if let Some(channel) = self.channel.as_mut() {
            let metrics = channel.metrics.get_or_insert_with(Default::default);
            metrics.gospels_spanish = Some(metrics.gospels_spanish.unwrap_or(0) + 1);
            self.save();
        }
    }

    pub fn add_channel_metrics_gospel_german(&mut self) {
        if let Some(channel) = self.channel.as_mut() {
            let metrics = channel.metrics.get_or_insert_with(Default::default);
            metrics.gospels_german = Some(metrics.gospels_german.unwrap_or(0) + 1);
            self.save();
        }
    }

    pub fn add_channel_metrics_scriptures(&mut self) {
        if let Some(scriptures) = self
            .channel
            .as_mut()
            .and_then(|chn| chn.metrics.as_mut())
            .and_then(|mtr| mtr.scriptures.as_mut())
        {
            *scriptures += 1;
            self.save();
        }

        if let Some(channel) = self.channel.as_mut() {
            let metrics = channel.metrics.get_or_insert_with(Default::default);
            metrics.scriptures = Some(metrics.scriptures.unwrap_or(0) + 1);
            self.save();
        }
    }
}
