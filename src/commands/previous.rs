use crate::helpers::statics::{BIBLES, REPLY_CHARACTER_LIMIT};
use crate::helpers::{response_builder::ResponseBuilder, Config};
use bible::scripture::bible::Bible;

pub async fn previous(display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message = "Previous Help: Responds with the previous verses in order, based on the last verse referenced, with the specified translation. You can optionally specify the number of verses you would like returned. Usage: !previous | !previous 2";
    if params
        .get(0)
        .map_or(false, |p| p == "?" || p.to_lowercase() == "help")
    {
        return Some(help_message.to_string());
    }

    let mut config = Config::load(&display_name);
    let verses_count = params
        .get(0)
        .and_then(|s| s.parse::<usize>().ok())
        .map(|number| number.clamp(1, 10))
        .unwrap_or(1);

    config
        .get_last_verse_and_translation()
        .and_then(|(last_verse, translation)| {
            BIBLES
                .get(&translation)
                .and_then(|bible_arc| {
                    let bible: &Bible = &*bible_arc;
                    let verses = bible.get_previous_scripture(&last_verse, verses_count);

                    match verses.is_empty() {
                        true => None,
                        false => {
                            let adjusted_character_limit =
                                *REPLY_CHARACTER_LIMIT - (display_name.len() + 1);
                            let response_output = ResponseBuilder::build(
                                &verses,
                                adjusted_character_limit,
                                &translation,
                            );
                            config.set_last_verse(&verses.first().unwrap().reference);
                            config.add_account_metrics_scriptures();

                            Some(response_output.truncated)
                        }
                    }
                })
                .or_else(|| {
                    eprintln!("No Bible version found for translation");
                    None
                })
        })
        .or_else(|| {
            eprintln!("No verse or translation available");
            None
        })
}
