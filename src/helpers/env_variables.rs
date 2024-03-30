use crate::helpers::print_color::PrintCommand;
use dotenv::dotenv;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{env, fs};

pub(crate) fn get_env_variable(env_var_key: &str, default_value: &str) -> String {
    let env_file_path = ".env";
    // TODO!  BUG  There is a bug that if anything is missing it will overwrite with a single value.
    // Attempt to load environment variables from .env file
    if dotenv().is_err() {
        match File::create(env_file_path) {
            Ok(_) => {
                match fs::canonicalize(env_file_path) {
                    Ok(path) => {
                        let mut path_str = path.to_string_lossy().to_string();
                        // Windows-specific path format adjustment
                        if cfg!(target_os = "windows") {
                            path_str = path_str.trim_start_matches(r"\\?\").to_string();
                        }
                        PrintCommand::Error
                            .print_message("Created missing .env file at", &path_str);
                    }
                    Err(_) => {
                        eprintln!("ERROR:  Failed to resolve the full path of the .env file!")
                    }
                }
            }
            Err(_) => {
                eprintln!("Failed to create .env file!");
                return default_value.to_string();
            }
        }
    }

    // Retrieve the value of the environment variable
    match env::var(env_var_key) {
        Ok(key_value) => key_value,
        Err(_) => {
            PrintCommand::Issue.print_message(
                &format!(
                    "{} variable missing from .env file! Defaulting to",
                    env_var_key
                ),
                default_value,
            );

            match OpenOptions::new().append(true).open(".env") {
                Ok(mut file) => {
                    let contents = fs::read_to_string(env_file_path).unwrap_or_default();
                    let newline = if !contents.is_empty() && !contents.ends_with('\n') {
                        "\n"
                    } else {
                        ""
                    };

                    let env_var_line = format!("{}{}={}\n", newline, env_var_key, default_value);
                    if file.write_all(env_var_line.as_bytes()).is_err() {
                        eprintln!("Failed to write to .env file!");
                    }
                }
                Err(_) => eprintln!("Failed to open .env file!"),
            }
            default_value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_get_env_variable_creates_and_updates_env_file() {
        let env_var_key = "TEST_ENV_VAR";
        let default_value = "default_value";
        let value = get_env_variable(env_var_key, default_value);
        // Assert that the function returns the default value when the env var is not set
        assert_eq!(value, default_value);
    }
}
