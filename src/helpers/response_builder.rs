use bible::scripture::bible::Verse;

pub struct ResponseOutput {
    pub truncated: String,
    pub remainder: String,
}

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn build(
        verses: &Vec<Verse>,
        total_length: usize,
        bible_name_to_use: &str,
    ) -> ResponseOutput {
        if verses.is_empty() {
            return ResponseOutput {
                truncated: String::new(),
                remainder: String::new(),
            };
        }

        let last_verse = verses.last().unwrap();
        let start_verse = verses.first().unwrap().verse;
        let end_verse = last_verse.verse;
        let abbreviation = last_verse
            .abbreviation
            .split(':')
            .next()
            .unwrap_or_default();

        let scripture_reference = if start_verse == end_verse {
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
            let adjusted_length = total_length - scripture_reference.len() - 6; // Adjusting length for " ... - "
            let break_point = scriptures
                .char_indices()
                .take_while(|&(idx, _)| idx <= adjusted_length)
                .filter(|&(_, c)| c == ' ')
                .map(|(idx, _)| idx)
                .last()
                .unwrap_or(adjusted_length);

            ResponseOutput {
                truncated: format!(
                    "{}... - {}",
                    scriptures[..break_point].trim_end(),
                    scripture_reference
                ),
                remainder: scriptures[break_point..].trim_start().to_string(),
            }
        } else {
            ResponseOutput {
                truncated: scripture_full,
                remainder: String::new(),
            }
        }
    }
}
