use crate::helpers::print_color::PrintCommand;
use dotenv::dotenv;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;

pub(crate) fn get_env_variable(env_var_key: &str, default_value: &str) -> String {
    let env_file_path = ".env";

    // Attempt to load environment variables from .env file
    if dotenv().is_err() {
        if File::create(env_file_path).is_ok() {
            PrintCommand::Error.print_message("Created missing .env file at", env_file_path);
        } else {
            eprintln!("Failed to create .env file!");
            return default_value.to_string();
        }
    }

    // Retrieve the value of the environment variable or append the default if missing
    match env::var(env_var_key) {
        Ok(value) => value,
        Err(_) => {
            let contents = fs::read_to_string(env_file_path).unwrap_or_default();

            // Check if the key is already defined in the .env file
            if contents.contains(&format!("{}=", env_var_key)) {
                // Variable key is present but empty
                return default_value.to_string();
            }

            // If the variable is not defined, append it to the .env file
            match OpenOptions::new()
                .write(true)
                .append(true)
                .open(env_file_path)
            {
                Ok(mut file) => {
                    let newline = if !contents.is_empty() && !contents.ends_with('\n') {
                        "\n"
                    } else {
                        ""
                    };
                    let env_var_line = format!("{}{}={}\n", newline, env_var_key, default_value);

                    if file.write_all(env_var_line.as_bytes()).is_ok() {
                        PrintCommand::Issue.print_message(
                            &format!(
                                "{} variable missing from .env file! Defaulting to",
                                env_var_key
                            ),
                            default_value,
                        );
                    } else {
                        eprintln!("Failed to write to .env file!");
                    }
                }
                Err(_) => {
                    eprintln!("Failed to open .env file!");
                }
            }
            default_value.to_string()
        }
    }
}
