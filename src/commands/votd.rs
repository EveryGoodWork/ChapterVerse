use crate::helpers::statics::{BIBLES, DEFAULT_TRANSLATION, REPLY_CHARACTER_LIMIT};
use crate::helpers::{
    response_builder::ResponseBuilder, webscraper::fetch_verse_of_the_day, Config,
};
use bible::scripture::bible::Bible;

pub async fn votd(channel: &str, display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message = "Verse of the Day (VOTD) Help: Retrieves the daily verse from an external source or allows you to set it manually. Use 'auto' to reset it to use the external source. Usage: !votd | !votd gen 1:1 | !votd auto";
    let auto_message = "Verse of the Day reset it to use the external source.";
    match params.get(0) {
        Some(p) if p == "?" || p.to_lowercase() == "help" => return Some(help_message.to_string()),
        Some(a) if a.to_lowercase() == "auto" => {
            let mut config = Config::load(&display_name);
            config.set_votd(None);
            return Some(auto_message.to_string());
        }
        Some(_) => {
            let mut config = Config::load(&display_name);
            let reference = params.join(" ");
            let translation = config
                .get_translation()
                .unwrap_or_else(|| DEFAULT_TRANSLATION.to_string());

            if let Some(bible_arc) = BIBLES.get(&translation) {
                let bible: &Bible = &*bible_arc;
                let verses = bible.get_scripture(&reference);

                if verses.is_empty() {
                    Some(format!("Invalid verse reference."))
                } else {
                    let adjusted_character_limit =
                        *REPLY_CHARACTER_LIMIT - (&display_name.len() + 1);
                    let response_output =
                        ResponseBuilder::build(&verses, adjusted_character_limit, &translation);

                    config.set_votd(Some(reference));
                    Some(format!(
                        "Verse of the day for {} channel, manually set to: {}",
                        &display_name, response_output.truncated
                    ))
                }
            } else {
                Some("No Bible version found for translation.".to_string())
            }
        }
        None => {
            let mut config = Config::load(&channel);

            let votd_reference: Option<String> = match config.get_votd() {
                Some(s) => Some(s),
                None => match fetch_verse_of_the_day().await {
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            };

            let votd_scripture = match votd_reference {
                Some(ref reference) => {
                    println!("Verse of the Day: {}", reference);
                    let translation = config
                        .get_translation()
                        .unwrap_or_else(|| DEFAULT_TRANSLATION.to_string());

                    if let Some(bible_arc) = BIBLES.get(&translation) {
                        let bible: &Bible = &*bible_arc;
                        let verses = bible.get_scripture(reference);

                        if verses.is_empty() {
                            None
                        } else {
                            let adjusted_character_limit =
                                *REPLY_CHARACTER_LIMIT - (&display_name.len() + 1);
                            let response_output = ResponseBuilder::build(
                                &verses,
                                adjusted_character_limit,
                                &translation,
                            );
                            config.set_last_verse(&verses.last().unwrap().reference);
                            config.add_account_metrics_scriptures();

                            Some(response_output.truncated)
                        }
                    } else {
                        eprintln!("No Bible version found for translation");
                        None
                    }
                }
                None => {
                    println!("Error: No Verse of the Day found");
                    None
                }
            };
            votd_scripture
        }
    }
}
