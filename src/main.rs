use helpers::import_bibles::start_bible_import;

use crate::helpers::env_variables::get_env_variable;
use crate::helpers::print_color::PrintCommand;
use std::{env, fs};

mod helpers;
mod scripture;

fn main() {
    PrintCommand::System.print_message("ChapterVerse", "Jesus is Lord!");
    PrintCommand::Issue.print_message("Version", env!("CARGO_PKG_VERSION"));
    PrintCommand::System.print_message("What is the Gospel?", "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"");

    let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");

    let bibles_directory = env::current_dir()
        .expect("Failed to get current directory")
        .join(import_bibles_path);

    PrintCommand::System.print_message("Bibles Directory", &bibles_directory.to_string_lossy());
    
    let files = fs::read_dir(&bibles_directory).expect("Failed to read directory");

    for file in files {
        let entry = file.expect("Failed to read entry");
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("csv") {
            match start_bible_import(&entry.path().to_string_lossy()) {
                Ok(imported_bible) => {
                    PrintCommand::System.print_message("Bible Imported", &entry.path().display().to_string());
                    PrintCommand::Info.print_message("Number of verses imported", &imported_bible.len().to_string());
                    PrintCommand::Info.print_message("2 Timothy 3:16", &imported_bible.get_scripture("55:3:16").0.then(|| imported_bible.get_scripture("55:3:16").1).unwrap_or_else(|| "Verse not found".to_string()));



                },
                Err(err) => {
                    println!("Error running import: {}", err);
                }
            }
        }
    }
    
}
