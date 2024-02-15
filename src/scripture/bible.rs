// use std::collections::HashMap;

// #[derive(Debug, Clone, PartialEq)]
// struct Bible {
//     id: String,
//     book_id: u8,
//     chapter: u8,
//     verse: u8,
//     text: String,
// }

// struct ScriptureIndex {
//     index: HashMap<String, Bible>,
// }

// impl ScriptureIndex {
//     fn new() -> Self {
//         ScriptureIndex {
//             index: HashMap::new(),
//         }
//     }

//     fn insert(&mut self, scripture: Bible) {
//         self.index.insert(scripture.id.clone(), scripture);
//     }

//     fn get(&self, id: &str) -> Option<&Bible> {
//         self.index.get(id)
//     }
// }
