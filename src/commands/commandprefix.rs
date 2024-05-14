use crate::helpers::Config;

pub async fn commandprefix(display_name: &str, params: Vec<String>) -> (Option<String>, char) {
    let help_message = "CommandPrefix Help: Displays the current command prefix (default: '!'). Specify a character to set a new prefix. IMPORTANT: Remember your new prefix as you will need it to make further changes, however you can reset it on another from another chat running ChapterVerse if needed. Usage: !commandprefix | !commandprefix #";

    let forbidden_chars = ['.', '/', '%', '?'];
    let config = Config::load(&display_name);
    let prefix = config.get_command_prefix();

    let message = params.get(0).map_or_else(
        || Some(format!("The command prefix character is: {}", prefix)),
        |p| {
            let param = p.to_lowercase();
            if param == "?" || param == "help" {
                Some(help_message.to_string())
            } else {
                let new_prefix = param.chars().next().unwrap_or('!');
                if forbidden_chars.contains(&new_prefix) {
                    Some(format!(
                        "Error: The character '{}' is forbidden as a command prefix. Forbidden characters are: {}",
                        new_prefix, forbidden_chars.iter().collect::<String>()
                    ))
                } else {
                    let mut config = Config::load(&display_name);
                    config.set_command_prefix(&new_prefix);
                    Some(format!("The command prefix character is: {}", new_prefix))
                }
            }
        },
    );
    let config = Config::load(&display_name);
    let changed_prefix = config.get_command_prefix();

    println!(
        "{} '{}'",
        &message.clone().unwrap().to_string(),
        &changed_prefix
    );
    (message, changed_prefix)
}
