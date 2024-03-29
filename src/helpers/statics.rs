use crate::helpers::env_variables::get_env_variable;
use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};

lazy_static! {
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
