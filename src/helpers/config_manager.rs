use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};
use std::path::PathBuf;
use toml;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    settings: HashMap<String, String>,
}

pub struct ConfigManager {
    config: Config,
    path: PathBuf,
}

impl ConfigManager {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let config: Config = toml::from_str(&content).unwrap_or_default();
        ConfigManager { config, path }
    }

    fn save(&self) {
        let toml_string = toml::to_string(&self.config).unwrap();
        fs::write(&self.path, toml_string).expect("Failed to write to file");
    }

    pub fn get_string(&mut self, key: &str, default: &str) -> String {
        if !self.config.settings.contains_key(key) {
            self.config
                .settings
                .insert(key.to_string(), default.to_string());
            self.save();
        }
        self.config.settings[key].clone()
    }
    pub fn set_string(&mut self, key: &str, value: &str) {
        self.config
            .settings
            .insert(key.to_string(), value.to_string());
        self.save();
    }
}
