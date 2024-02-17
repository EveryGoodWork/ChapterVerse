use std::collections::HashMap;

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Verse {
    id: String,
    book_id: u8,
    chapter: u8,
    verse: u8,
    scripture: String,
}

pub struct Bible {
    index: HashMap<String, Verse>,
}
impl Bible {
    pub fn new() -> Self {
        Bible {
            index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, scripture: Verse) {
        self.index.insert(scripture.id.clone(), scripture);
    }
    // pub fn get(&self, id: &str) -> Option<&Verse> {
    //     self.index.get(id)
    // }
    pub fn get_scripture(&self, id: &str) -> (bool, String) {
        if let Some(verse) = self.index.get(id) {
            (true, verse.scripture.clone())
        } else {
            (false, String::new())
        }
    }
    pub fn len(&self) -> usize {
        self.index.len()
    }
}
