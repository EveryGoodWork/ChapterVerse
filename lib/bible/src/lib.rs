pub mod scripture;

pub mod csv_import {
    use crate::scripture::bible::{Bible, Verse};
    use std::error::Error;

    pub fn bible_import(bible_import_path: &str) -> Result<Bible, Box<dyn Error>> {
        let mut csv_reader = csv::Reader::from_path(bible_import_path)?;
        let mut bible = Bible::new(); // Instantiate ScriptureIndex

        for result in csv_reader.deserialize() {
            let record: Verse = result?;
            bible.insert(record); // Insert each Bible record into the index
        }
        Ok(bible) // Return the populated ScriptureIndex
    }
}
