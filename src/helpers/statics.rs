use crate::helpers::config::Config;
use crate::helpers::env_variables::get_env_variable;
use crate::helpers::Metrics;
use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use chrono::{DateTime, Local, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};
use tokio::sync::RwLock;

pub fn initialize_statics() {
    // Access each lazy_static to trigger its initialization.
    // let _ = &*TWITCH_ACCOUNT;
    let _ = &*START_DATETIME_UTC;
    let _ = &*START_DATETIME_LOCAL;
    // let _ = &*BIBLES_REGEX;
    // let _ = &*CHANNELS_TO_JOIN;
    // let _ = &*METRICS;
    // let _ = &*BIBLES;
}
// Important Note: lazy_static's are not loaded until the first time they are called.
lazy_static! {

    pub static ref CHANNELS_PER_LISTENER: usize = 5;
    // TODO! Remove the debug deduction for the (7.7118ms) - 10 characters
    pub static ref  REPLY_CHARACTER_LIMIT: usize = 500 - 10;
    // The only reason we use KJV as default is because it's free to use from copywrite restrictions.
    pub static ref  DEFAULT_TRANSLATION: String = "KJV".to_string();

    pub static ref TWITCH_ACCOUNT: String = get_env_variable("TWITCHACCOUNT", "twitchusername");
    pub static ref START_DATETIME_UTC: DateTime<Utc> = Utc::now();
    pub static ref START_DATETIME_UTC_STRING: String = START_DATETIME_UTC.format("%Y/%m/%d %H:%M UTC").to_string();


pub static ref START_DATETIME_LOCAL: DateTime<Local> = Local::now();
pub static ref START_DATETIME_LOCAL_STRING: String = {
    let timezone_str = match START_DATETIME_LOCAL.format("%Z").to_string().as_str() {
        "-07:00" => "PDT",
        "-08:00" => "PST",
        _ => "",
    };
    format!("{} {}", START_DATETIME_LOCAL.format("%Y/%m/%d %H:%M").to_string(), timezone_str)
};

pub static ref BIBLES_REGEX: Regex = {
    let bible_names = BIBLES.keys().map(|name| name.as_str()).collect::<Vec<&str>>().join("|");
    Regex::new(&format!(r"(?i)\b({})\b", bible_names)).expect("Invalid regex pattern")
};

#[derive(Debug)]
pub static ref CHANNELS_TO_JOIN: Vec<String> = Config::get_channels();

//pub static ref METRICS: Arc<Mutex<metrics::Metrics>> = Arc::new(Mutex::new(metrics::Metrics::default()));
pub static ref METRICS: Arc<RwLock<Metrics>> = Arc::new(RwLock::new(Metrics::default()));


pub static ref BIBLES: Arc<HashMap<String, Arc<Bible>>> = {

            let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");

            let bibles_directory = match env::current_dir().map(|dir| dir.join(import_bibles_path)) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("Error getting current directory: {}", e);
                    return Arc::new(HashMap::new());
                }
            };

            let mut bibles = HashMap::new();

            let files = match fs::read_dir(bibles_directory) {
                Ok(files) => files,
                Err(e) => {
                    println!("Error reading bibles directory: {}", e);
                    return Arc::new(HashMap::new());
                }
            };

            for file in files {
                let entry = match file {
                    Ok(entry) => entry,
                    Err(e) => {
                        println!("Error reading file in directory: {}", e);
                        continue; // Skip to the next iteration
                    }
                };

                if entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("csv")
                {
                    let file_stem = entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string()
                        .to_uppercase();
                    let file_path = entry.path().to_string_lossy().to_string();
                    match bible_import(&entry.path().to_string_lossy()) {
                        Ok(imported_bible) => {
                            bibles.insert(file_stem, Arc::new(imported_bible));
                        }
                        Err(err) => {
                            println!("Error running import for file '{}': {}", file_path, err);
                        }
                    }
                }
            }

            Arc::new(bibles)
        };
    }

pub fn find_bible(input: String, default: &String) -> String {
    BIBLES_REGEX
        .find(&input)
        .map(|m| m.as_str().to_uppercase())
        .unwrap_or_else(|| default.to_string())
}

pub fn avaialble_bibles() -> String {
    BIBLES
        .keys()
        .map(|key| key.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn get_running_time() -> String {
    let duration = Utc::now().signed_duration_since(*START_DATETIME_UTC);
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let running_time = format!("{:02}d {}h {}m", days, hours, minutes);
    running_time
}
