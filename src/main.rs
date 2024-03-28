use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use helpers::env_variables::get_env_variable;
use helpers::print_color::PrintCommand;
use tokio::sync::mpsc;

use futures::future::pending;
use twitch::chat::Listener;
use twitch::common::message_data::MessageData;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};
use twitch::chat::client::WebSocketState;

mod helpers;

lazy_static! {
    static ref BIBLES: Arc<HashMap<String, Arc<Bible>>> = {
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
#[allow(unused)]
fn get_bibles_names() -> String {
    BIBLES.keys().cloned().collect::<Vec<_>>().join(", ")
}
#[allow(unused)]
fn get_specific_bible(bible_name: &str) -> Option<Arc<Bible>> {
    let bibles = Arc::clone(&BIBLES); // Clone the Arc for thread-safe access
    let lookup_name = bible_name.to_uppercase(); // Convert the lookup name to lowercase
    bibles.get(&lookup_name).cloned()
}
#[tokio::main]
async fn main() {
    PrintCommand::System.print_message("ChapterVerse", "Jesus is Lord!");
    PrintCommand::Issue.print_message("Version", env!("CARGO_PKG_VERSION"));
    PrintCommand::Info.print_message("What is the Gospel?", "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"");
    for (bible_name, bible_arc) in BIBLES.iter() {
        let bible: &Bible = &*bible_arc; // Dereference the Arc and immediately borrow the result
        let scripture = match bible.get_scripture("2 Timothy 3:16") {
            Some(verse) => format!("{}", verse.scripture),
            None => "Verse not found".to_string(),
        };
        PrintCommand::Info.print_message(&format!("{}, 2 Timothy 3:16", bible_name), &scripture);
    }

    // TODO:  Create a config files to pull these from, each channel gets it's own file.
    let channels_to_join = vec!["chapterverse".to_string(), "missionarygamer".to_string()];

    let (tx, rx) = mpsc::unbounded_channel::<MessageData>();
    println!("Trying to connect");

    let listener = Arc::new(Listener::new(tx));
    // Assuming `channels_to_join` is cloned or moved into the async block appropriately
    let channels_clone = channels_to_join.clone();
    // Spawn a task to manage connection, listening, and reconnection
    tokio::spawn(async move {
        println!("Inside  tokio::spawn");
        let listener_clone = Arc::clone(&listener);
        loop {
            println!("Inside Loop");
            // Clone `listener_clone` for each loop iteration to avoid moving the original `Arc`
            let loop_listener_clone = Arc::clone(&listener_clone);
            // Attempt to connect
            match loop_listener_clone.connect().await {
                Ok(_) => println!("Successfully connected."),
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue; // Retry connection
                }
            }

            // Join channels after successful connection
            for channel in &channels_clone {
                match listener_clone.join_channel(channel).await {
                    Ok(_) => println!("Successfully joined channel: {}", channel),
                    Err(e) => eprintln!("Failed to join channel {}: {}", channel, e),
                }
            }

            // Listen for messages or disconnection events
            while listener_clone.get_state() != WebSocketState::Disconnected {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // If disconnected, wait before attempting to reconnect
            //THIS IS NOT WORKING WHEN IT"S DISCONNECTED.
            //Error receiving message: Io(Os { code: 10054, kind: ConnectionReset, message: "An existing connection was forcibly closed by the remote host." })
            println!("Disconnected, attempting to reconnect...");
        }
    });

    // Process received messages in another task
    let mut rx_clone = rx;
    tokio::spawn(async move {
        while let Some(message) = rx_clone.recv().await {
            // TODO! Add a preliminary scan to determine if there is potential scripture(s) in this message.
            // TODO! this will pull from a user preference variable
            let bible_name_to_use = "KJV";

            if let Some(bible_arc) = BIBLES.get(bible_name_to_use) {
                let bible: &Bible = &*bible_arc;
                let scripture_message = match bible.get_scripture(&message.text) {
                    Some(verse) => format!("{}", verse.scripture),
                    None => "Verse not found".to_string(),
                };
                PrintCommand::Info.print_message(
                    &format!(
                        "{}, {}",
                        bible_name_to_use,
                        message.display_name.unwrap_or_default()
                    ),
                    &scripture_message,
                );
            } else {
                eprintln!("Bible named '{}' not found.", bible_name_to_use);
            }

            match message.complete() {
                Ok(duration) => println!("Message processing duration: {:?}", duration),
                Err(e) => eprintln!("Error calculating duration: {}", e),
            }
        }
    });
    // This line will keep the program running indefinitely until it's killed manually (e.g., Ctrl+C).
    pending::<()>().await;
}

#[cfg(test)]
mod unittests {
    use super::*;
    // use the following command line to see the results of the test: cargo test -- --nocapture
    #[test]
    fn get_scripture() {
        for (bible_name, bible_arc) in BIBLES.iter() {
            let bible: &Bible = &*bible_arc; // Here you dereference the Arc and immediately borrow the result

            let message = match bible.get_scripture("2 Timothy 3:16") {
                Some(verse) => format!("{}", verse.scripture),
                None => "Verse not found".to_string(),
            };

            PrintCommand::Info.print_message(&format!("{}, 2 Timothy 3:16", bible_name), &message);
        }
    }
}
