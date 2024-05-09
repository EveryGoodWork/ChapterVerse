use crate::helpers::statics::{BIBLES, DEFAULT_TRANSLATION, REPLY_CHARACTER_LIMIT};
use crate::helpers::{
    response_builder::ResponseBuilder, webscraper::fetch_verse_of_the_day, Config,
};
use bible::scripture::bible::Bible;

pub async fn votd(display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message =
        "Verse of the Day (VOTD) Help: Retrieves daily verse from an external source. Usage: !votd";
    if params[0] == "?" || params[0].to_lowercase() == "help" {
        return Some(help_message.to_string());
    }

    let mut config = Config::load(&display_name);
    let votd_scripture = match fetch_verse_of_the_day().await {
        Ok(verse) => {
            println!("Verse of the Day: {}", verse);
            let translation = config
                .get_translation()
                .unwrap_or_else(|| DEFAULT_TRANSLATION.to_string());

            if let Some(bible_arc) = BIBLES.get(&translation) {
                let bible: &Bible = &*bible_arc;
                let verses = bible.get_scripture(&verse);

                if verses.is_empty() {
                    None
                } else {
                    let adjusted_character_limit =
                        *REPLY_CHARACTER_LIMIT - (&display_name.len() + 1);
                    let response_output =
                        ResponseBuilder::build(&verses, adjusted_character_limit, &translation);
                    config.set_last_verse(&verses.last().unwrap().reference);
                    config.add_account_metrics_scriptures();

                    Some(response_output.truncated)
                }
            } else {
                eprintln!("No Bible version found for translation");
                None
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            None
        }
    };

    votd_scripture
}
