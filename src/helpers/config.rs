use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error as IoError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub channel: Option<Channel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub channelname: Option<String>,
    pub temp: Option<String>,
}
#[derive(Debug, Clone)]
pub enum ConfigError {
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}
impl std::error::Error for ConfigError {}
impl Config {
    // Function to save the Config struct directly to a file
    pub fn save(&self) -> Result<(), IoError> {
        let toml_string = toml::to_string(&self)
            .map_err(|e| IoError::new(std::io::ErrorKind::Other, e.to_string()))?;

        fs::write("./channels/chapterverse_new.toml", toml_string)
    }
    // Function to create a new Config instance from a TOML file
    pub fn new() -> Self {
        let filepath = "./channels/chapterverse.toml";
        let content = fs::read_to_string(filepath).unwrap_or_else(|_| "".to_owned());
        toml::from_str(&content).unwrap_or_else(|_| Config { channel: None })
    }

    pub fn set_channel_name(&mut self, name: String) {
        let channel = self.channel.get_or_insert_with(|| Channel {
            channelname: None,
            temp: None,
        });
        channel.channelname = Some(name);
    }

    pub fn set_temp(&mut self, temp: String) {
        let channel = self.channel.get_or_insert_with(|| Channel {
            channelname: None,
            temp: None,
        });
        channel.temp = Some(temp);
    }

    pub fn temp(&self) -> Result<String, ConfigError> {
        self.channel
            .as_ref()
            .and_then(|channel| channel.temp.clone())
            .ok_or_else(|| ConfigError::MissingField("temp".to_string()))
    }

    pub fn channel_name(&self) -> Result<String, ConfigError> {
        self.channel
            .as_ref()
            .and_then(|channel| channel.channelname.clone())
            .ok_or_else(|| ConfigError::MissingField("channelname".to_string()))
    }
}
