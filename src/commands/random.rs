use crate::helpers::statics::{BIBLES, DEFAULT_TRANSLATION, REPLY_CHARACTER_LIMIT};
use crate::helpers::{response_builder::ResponseBuilder, Config};
use bible::scripture::bible::Bible;

pub async fn random(channel: &str, display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message =
        "Random Help: Retrieves a random verse from scripture using your preferred translation. Usage: !random";
    if params
        .get(0)
        .map_or(false, |p| p == "?" || p.to_lowercase() == "help")
    {
        return Some(help_message.to_string());
    }

    let mut config = Config::load(&display_name);
    let translation = config
        .get_translation()
        .unwrap_or_else(|| DEFAULT_TRANSLATION.to_string());

    if let Some(bible_arc) = BIBLES.get(&translation) {
        let bible: &Bible = &*bible_arc;
        let random_scripture = bible.random_scripture();

        if random_scripture.is_empty() {
            None
        } else {
            let adjusted_character_limit = *REPLY_CHARACTER_LIMIT - (&display_name.len() + 1);
            let response_output =
                ResponseBuilder::build(&random_scripture, adjusted_character_limit, &translation);
            config.set_last_verse(&random_scripture.last().unwrap().reference);
            config.add_account_metrics_scriptures();

            if !channel.eq_ignore_ascii_case(display_name) {
                Config::load(channel).add_channel_metrics_scriptures();
            } else {
                config.add_channel_metrics_scriptures();
            }

            Some(response_output.truncated)
        }
    } else {
        eprintln!("No Bible version found for translation");
        None
    }
}
