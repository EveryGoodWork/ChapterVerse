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

// Important Note: lazy_static's are not loaded until the first time they are called.
lazy_static! {

pub static ref TWITCH_ACCOUNT: String = get_env_variable("TWITCHACCOUNT", "twitchusername");
pub static ref GOSPEL: String = "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"".to_string();
pub static ref EVANGELIO: String = "El evangelio significa buenas nuevas! La mala noticia es que todos hemos pecado y merecemos la ira venidera. Pero Jesus, el Mesias, murio por nuestros pecados, fue sepultado y resucito al tercer dia segun las Escrituras. Ascendio a los cielos y esta sentado a la diestra del Padre. Jesus dijo: \"Yo soy el camino, la verdad y la vida. Nadie viene al Padre sino por mi. El tiempo se ha cumplido, y el reino de Dios se ha acercado; arrepentios y creed en el evangelio\".".to_string();
pub static ref EVANGELIUM: String = "Evangelium bedeutet Gute Nachricht! Die schlechte Nachricht ist, wir haben alle gesundigt und verdienen Gottes Zorn. Doch Jesus Christus starb fur unsere Sunden, wurde begraben und am dritten Tag auferweckt, nach der Bibel. Er fuhr in den Himmel auf und sitzt jetzt zur Rechten des Vaters. Jesus sagt: \"Ich bin der Weg, die Wahrheit und das Leben; niemand kommt zum Vater ausser durch mich.\" Die Zeit ist reif und das Reich Gottes ist nahe; kehrt um und glaubt an das Evangelium.".to_string();
pub static ref START_DATETIME_UTC: DateTime<Utc> = Utc::now();

pub static ref START_DATETIME_LOCAL: String = {
    let local_time = Local::now();
    let timezone_str = match local_time.format("%Z").to_string().as_str() {
        "-07:00" => "PDT",
        "-08:00" => "PST",
        _ => "",
    };
    format!("{} {}", local_time.format("%Y/%m/%d %H:%M").to_string(), timezone_str)
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
