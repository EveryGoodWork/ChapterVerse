use crate::helpers::{statics::BIBLES, Config};

pub async fn translation(
    display_name: &str,
    params: Vec<String>,
    available_bibles: fn() -> String,
) -> Option<String> {
    let mut config = Config::load(&display_name);
    let help_message = "Translation Help:  Sets your default Bible translation for scripture queries. If a different translation is specified in a future query, it defaults back to the set translation on subsequent requests.  Usage:  !translation web";
    let invalid_parameter_message = "Please provide a valid translation identifier. Example: '!translation web'. Available translations:";
    let command_success = "Your preferred translation set to:";

    if params.is_empty()
        || params
            .get(0)
            .map_or(false, |p| p == "?" || p.to_lowercase() == "help")
    {
        return Some(help_message.to_string());
    }

    if params.get(0).map_or(false, |p| !p.is_ascii()) {
        return Some(format!(
            "{} {}",
            invalid_parameter_message,
            available_bibles()
        ));
    }

    let translation = params[0].to_uppercase();
    if BIBLES.contains_key(&translation) {
        config.preferred_translation(&translation);
        Some(format!(
            "{} {}",
            command_success,
            config.get_translation().unwrap()
        ))
    } else {
        Some(format!(
            "{} {}.",
            invalid_parameter_message,
            available_bibles()
        ))
    }
}
