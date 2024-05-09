use crate::helpers::statics::{BIBLES, REPLY_CHARACTER_LIMIT};
use crate::helpers::{response_builder::ResponseBuilder, Config};
use bible::scripture::bible::Bible;

pub async fn next(display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message =
        "Next Help: Responds with the next verse in order, based on the last verse referenced, with the specified translation. You can optionally specify the number of verses you would like returned. Usage: !next | !next 2";
    if params[0] == "?" || params[0].to_lowercase() == "help" {
        return Some(help_message.to_string());
    }

    let mut config = Config::load(&display_name);
    let verses = params
        .get(0)
        .and_then(|s| s.parse::<usize>().ok())
        .map(|number| number.clamp(1, 10))
        .unwrap_or(1);

    if let Some((last_verse, translation)) = config.get_last_verse_and_translation() {
        if let Some(bible_arc) = BIBLES.get(&translation) {
            let bible: &Bible = &*bible_arc;
            let verses = bible.get_next_scripture(&last_verse, verses);

            if verses.is_empty() {
                None
            } else {
                let adjusted_character_limit = *REPLY_CHARACTER_LIMIT - (&display_name.len() + 1);
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
    } else {
        eprintln!("No verse or translation available");
        None
    }
}
