use crate::helpers::webscraper::fetch_verse_of_the_day;

pub async fn votd() -> Option<String> {
    match fetch_verse_of_the_day().await {
        Ok(verse) => {
            println!("Verse of the Day: {}", verse);
            Some(verse)
        }
        Err(e) => {
            println!("Error: {}", e);
            None
        }
    }
}
