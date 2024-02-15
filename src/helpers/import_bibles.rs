use crate::{helpers::print_color::PrintCommand, scripture::bible::{Bible, Verse}};

use std::error::Error;
pub fn start_bible_import(bible_import_path: &str) -> Result<Bible, Box<dyn Error>> {
    PrintCommand::System.print_message("Bible import start", bible_import_path);

    let mut csv_reader = csv::Reader::from_path(bible_import_path)?;
    let mut scripture_index = Bible::new(); // Instantiate ScriptureIndex

    for result in csv_reader.deserialize() {
        let record: Verse = result?;
        scripture_index.insert(record); // Insert each Bible record into the index
    }

    // Confirm completion before returning the populated index
    PrintCommand::System.print_message("Bible import completed", bible_import_path);
    Ok(scripture_index) // Return the populated ScriptureIndex
}

#[cfg(test)]
mod tests {
    use crate::helpers::env_variables::get_env_variable;

    use super::*;
    use std::{env, fs, path::PathBuf};

    fn execution_dir() -> PathBuf {
        let mut exe_dir = env::current_exe()
            .expect("Failed to get the current executable path")
            .parent()
            .expect("Failed to get the executable's directory")
            .to_path_buf();
        if exe_dir.ends_with("deps") {
            exe_dir.pop(); // Remove the 'deps' component to move up to the expected directory
        }
        exe_dir
    }

    #[test]
    fn ensure_at_least_one_csv_in_bibles() {
        // Construct the path to the 'bibles' directory relative to the root directory
        let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");
        let bibles_dir = execution_dir().join(import_bibles_path);
        let bibles_dir_str = bibles_dir.to_str().unwrap_or("[Invalid UTF-8 in path]");
        PrintCommand::System.print_message(
            "bibles_dir",
            &format!("{}: {:?}", "bibles_dir", bibles_dir_str),
        );

        let entries = fs::read_dir(&bibles_dir).expect("Failed to read bibles directory");
        let mut csv_files = 0;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                csv_files += 1;
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                PrintCommand::System
                    .print_message("Bible", &format!("{}: {:?}", "csv_file_found", file_name));
            }
        }

        assert!(csv_files > 0, "No CSV files found in bibles directory");
    }

    #[test]
    fn test_single_bible_row_with_dynamic_search() {
        let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");
        let bibles_dir = execution_dir().join(import_bibles_path);
        let entries = fs::read_dir(&bibles_dir).expect("Failed to read bibles directory");

        let mut test_bible_path = None;

        for entry in entries.filter_map(Result::ok) {
            if entry.path().file_name().and_then(|s| s.to_str()) == Some("WEB.csv") {
                // This checks specifically for the file named "WEB.csv"
                test_bible_path = Some(entry.path());
                break; // Breaks once the WEB.csv file is found
            }
        }
        let test_bible_path_buf = test_bible_path.expect("No CSV files found in bibles directory");

        // Convert PathBuf to &str for start_bible_import function
        let test_bible_path_str = test_bible_path_buf.to_str().unwrap();



        // Now that we have the bible CSV path dynamically, we proceed with the test logic
        let verse_content = "The grace of the Lord Jesus Christ be with all the saints. Amen.";
        let bible = start_bible_import(test_bible_path_str).unwrap();
        let verse_key = "66:22:21".to_string();
        let verse = match bible.get_scripture(&verse_key) {
            (true, verse) => verse,
            (false, _) => panic!("Verse not found"),
        };
        
        println!("{}", verse_content);
        println!("{}", verse);
        assert_eq!(verse, verse_content, "Verse content does not match expected value");
    }
}
