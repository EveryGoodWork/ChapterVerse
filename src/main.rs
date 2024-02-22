use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use helpers::env_variables::get_env_variable;
use helpers::print_color::PrintCommand;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{env, fs, io};

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

        let files = match fs::read_dir(&bibles_directory) {
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
                    .to_string();
                match bible_import(&entry.path().to_string_lossy()) {
                    Ok(imported_bible) => {
                        bibles.insert(file_stem, Arc::new(imported_bible));
                    }
                    Err(err) => {
                        println!("Error running import: {}", err);
                    }
                }
            }
        }

        Arc::new(bibles)
    };
}

fn get_bibles_names() -> String {
    BIBLES.keys().cloned().collect::<Vec<_>>().join(", ")
}

fn get_specific_bible(bible_name: &str) -> Option<Arc<Bible>> {
    let bibles = Arc::clone(&BIBLES); // Clone the Arc for thread-safe access
    bibles.get(bible_name).cloned()
}

fn main() {
    PrintCommand::System.print_message("ChapterVerse", "Jesus is Lord!");
    PrintCommand::Issue.print_message("Version", env!("CARGO_PKG_VERSION"));
    PrintCommand::System.print_message("What is the Gospel?", "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"");

    //Temp commandline to confirm lookup of scripture is working
    let mut bible_name = String::new();
    let mut scripture_reference = String::new();

    print!("Enter Bible version ({}): ", get_bibles_names());
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut bible_name).unwrap();
    bible_name = bible_name.trim().to_string();

    print!("Enter Scripture reference (e.g., for 2 Tim 3:16 use '55:3:16'): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut scripture_reference).unwrap();
    scripture_reference = scripture_reference.trim().to_string();

    println!("{}: {}", bible_name, scripture_reference);
    if let Some(bible_arc) = get_specific_bible(&bible_name) {
        let bible: &Bible = &*bible_arc;
        let (found, scripture) = bible.get_scripture(&scripture_reference);
        if found {
            println!("{}: {}", scripture_reference, scripture);
        } else {
            println!("Verse not found");
        }
    } else {
        println!("Bible version not found: {}", bible_name);
    }
}

#[cfg(test)]
mod unittests {
    use super::*;

    #[test]
    fn get_scripture() {
        for (bible_name, bible_arc) in BIBLES.iter() {
            let bible: &Bible = &*bible_arc; // Here you dereference the Arc and immediately borrow the result

            PrintCommand::Info.print_message(
                &format!("{}, 2 Timothy 3:16", bible_name),
                &bible
                    .get_scripture("55:3:16")
                    .0
                    .then(|| bible.get_scripture("55:3:16").1)
                    .unwrap_or_else(|| "Verse not found".to_string()),
            );
        }
    }
}
