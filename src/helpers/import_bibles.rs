use crate::helpers::print_color::PrintCommand;

pub fn start_bible_import(bible_import_path: &str) {
    PrintCommand::System.print_message("Bible import start", bible_import_path);
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
}
