use bible::scripture::bible::Verse;

pub struct ResponseOutput {
    pub truncated: String,
    pub remainder: String,
    pub last_verse: String,
}

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn build(
        verses: &Vec<Verse>,
        total_length: usize,
        bible_name_to_use: &str,
    ) -> ResponseOutput {
        let last_verse = verses.last().unwrap();
        let start_verse = verses.first().unwrap().verse;
        let end_verse = last_verse.verse;
        let reference = last_verse.reference.split(':').next().unwrap_or_default();
        let abbreviation = last_verse
            .abbreviation
            .split(':')
            .next()
            .unwrap_or_default();

        let scripture_reference = if start_verse == end_verse {
            format!("{}:{} {}", reference, start_verse, bible_name_to_use)
        } else {
            format!(
                "{}:{}-{} {}",
                reference, start_verse, end_verse, bible_name_to_use
            )
        };

        let scripture_reference_abbreviation = if start_verse == end_verse {
            format!("{}:{} {}", abbreviation, start_verse, bible_name_to_use)
        } else {
            format!(
                "{}:{}-{} {}",
                abbreviation, start_verse, end_verse, bible_name_to_use
            )
        };

        let scriptures = verses
            .iter()
            .map(|verse| verse.scripture.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let scripture_full = format!("{} - {}", scriptures, scripture_reference);

        if scripture_full.len() > total_length {
            // -18 is for having " !next to continue" included in the message if the verse gets cut
            // off
            let adjusted_length = total_length - scripture_reference_abbreviation.len() - 18;
            let mut scripture = String::new();
            let mut last_verse = verses.last().unwrap();
            for verse in verses {
                if format!("{} {}", scripture, verse.scripture).len() < adjusted_length {
                    scripture += " ";
                    scripture += &verse.scripture;
                    last_verse = verse;
                } else {
                    break;
                }
            }
            let break_point = scriptures
                .char_indices()
                .take_while(|&(idx, _)| idx <= adjusted_length)
                .filter(|&(_, c)| c == ' ')
                .map(|(idx, _)| idx)
                .last()
                .unwrap_or(adjusted_length);
            // reused code here to make shortened scripture_reference_abbreviation accurate
            let scripture_reference_abbreviation = if start_verse == end_verse {
                format!("{}:{} {}", abbreviation, start_verse, bible_name_to_use)
            } else {
                format!(
                    "{}:{}-{} {}",
                    abbreviation, start_verse, last_verse.verse, bible_name_to_use
                )
            };

            ResponseOutput {
                truncated: format!(
                    "{} - {} !next to continue",
                    scripture,
                    scripture_reference_abbreviation
                ),
                remainder: scriptures[break_point..].trim_start().to_string(),
                last_verse: last_verse.reference.clone(),
            }
        } else {
            ResponseOutput {
                truncated: scripture_full,
                remainder: String::new(),
                last_verse: verses.last().unwrap().reference.clone(),
            }
        }
    }
}

