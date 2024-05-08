use regex::Regex;
use std::collections::HashMap;

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Verse {
    pub reference: String,
    pub abbreviation: String,
    pub book: u8,
    pub chapter: u8,
    pub verse: u8,
    pub scripture: String,
}

// #[derive(Default)]
pub struct Bible {
    scriptures: HashMap<String, Verse>,
    index: Vec<String>,
    regex: Regex,
}

impl Bible {
    pub fn new() -> Self {
        Self {
            scriptures: HashMap::new(),
            index: Vec::new(),
            regex: Regex::new(r"(?i)(\d?\s?[a-z]+\s?\d?)\s(\d+):(\d+)(?:-(\d+))?")
                .expect("Invalid regex pattern"),
        }
    }

    pub fn insert(&mut self, scripture: Verse) {
        self.scriptures
            .insert(scripture.reference.clone(), scripture.clone());
        self.index.push(scripture.reference.clone());
    }

    pub fn get_next_scripture(&self, current_reference: &str, verses: usize) -> Vec<Verse> {
        self.index
            .iter()
            .position(|r| r == current_reference)
            .and_then(|pos| self.index.get(pos + 1..pos + 1 + verses))
            .map_or(Vec::new(), |references| {
                references
                    .iter()
                    .flat_map(|reference| self.get_scripture(reference))
                    .collect()
            })
    }

    pub fn get_scripture(&self, reference: &str) -> Vec<Verse> {
        let mut verses = Vec::new();
        if let Some(caps) = self.regex.captures(reference) {
            let book_abbr = caps.get(1).map_or("", |m| m.as_str()).trim();
            let chapter = caps.get(2).map_or("", |m| m.as_str());
            let start_verse = caps
                .get(3)
                .map_or(0, |m| m.as_str().parse::<u8>().unwrap_or(0));
            let end_verse = caps.get(4).map_or(start_verse, |m| {
                m.as_str().parse::<u8>().unwrap_or(start_verse)
            });

            if start_verse > 0 && end_verse >= start_verse {
                let book_name = Self::get_bible_book_name(book_abbr);
                for verse_num in start_verse..=end_verse {
                    let formatted_ref = format!("{} {}:{}", book_name, chapter, verse_num);
                    if let Some(verse) = self.scriptures.get(&formatted_ref) {
                        verses.push(verse.clone());
                    }
                }
            }
        }
        verses
    }

    pub fn len(&self) -> usize {
        self.scriptures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get_bible_book_name(abbreviation: &str) -> &'static str {
        match abbreviation.to_lowercase().as_str() {
            "genesis" | "gen" | "ge" | "gn" => "Genesis",
            "exodus" | "ex" | "exod" | "exo" => "Exodus",
            "leviticus" | "lev" | "le" | "lv" => "Leviticus",
            "numbers" | "num" | "nu" | "nm" | "nb" => "Numbers",
            "deuteronomy" | "deut" | "de" | "dt" => "Deuteronomy",
            "joshua" | "josh" | "jos" | "jsh" => "Joshua",
            "judges" | "judg" | "jdg" | "jg" | "jdgs" => "Judges",
            "ruth" | "rth" | "ru" => "Ruth",
            "1 samuel" | "1 sam" | "1 sm" | "1 sa" | "1 s" | "i sam" | "i sa" | "1sam" | "1sa"
            | "1s" | "1st samuel" | "1st sam" | "first samuel" | "first sam" => "1 Samuel",
            "2 samuel" | "2 sam" | "2 sm" | "2 sa" | "ii sam" | "ii sa" | "2sam" | "2nd samuel"
            | "2nd sam" | "second samuel" | "second sam" => "2 Samuel",
            "1 kings" | "1 kgs" | "1 ki" | "1kgs" | "1kin" | "1ki" | "1k" | "i kgs" | "i ki"
            | "1st kings" | "1st kgs" | "first kings" | "first kgs" => "1 Kings",
            "2 kings" | "2 kgs" | "2 ki" | "2kgs" | "2kin" | "2ki" | "2k" | "ii kgs" | "ii ki"
            | "2nd kings" | "2nd kgs" | "second kings" | "second kgs" => "2 Kings",
            "1 chronicles" | "1 chron" | "1 chr" | "1 ch" | "1chron" | "1chr" | "1ch"
            | "i chron" | "i chr" | "i ch" | "1st chronicles" | "1st chron"
            | "first chronicles" | "first chron" => "1 Chronicles",
            "2 chronicles" | "2 chron" | "2 chr" | "2 ch" | "2chron" | "2chr" | "2ch"
            | "ii chron" | "ii chr" | "ii ch" | "2nd chronicles" | "2nd chron"
            | "second chronicles" | "second chron" => "2 Chronicles",
            "ezra" | "ezr" | "ez" => "Ezra",
            "nehemiah" | "neh" | "ne" => "Nehemiah",
            "esther" | "est" | "esth" | "es" => "Esther",
            "job" | "jb" => "Job",
            "psalm" | "psalms" | "ps" | "pslm" | "psa" | "psm" | "pss" => "Psalm",
            "proverbs" | "prov" | "pro" | "prv" | "pr" => "Proverbs",
            "ecclesiastes" | "eccles" | "eccle" | "ecc" | "ec" | "qoh" => "Ecclesiastes",
            "song of solomon" | "song of songs" | "sos" => "Song of Solomon",
            "isaiah" | "isa" | "is" => "Isaiah",
            "jeremiah" | "jer" | "je" | "jr" => "Jeremiah",
            "lamentations" | "lam" | "la" => "Lamentations",
            "ezekiel" | "ezek" | "eze" | "ezk" => "Ezekiel",
            "daniel" | "dan" | "da" | "dn" => "Daniel",
            "hosea" | "hos" | "ho" => "Hosea",
            "joel" | "jl" => "Joel",
            "amos" | "am" => "Amos",
            "obadiah" | "obad" | "ob" => "Obadiah",
            "jonah" | "jnh" | "jon" => "Jonah",
            "micah" | "mic" | "mc" => "Micah",
            "nahum" | "nah" | "na" => "Nahum",
            "habakkuk" | "hab" | "hb" => "Habakkuk",
            "zepheniah" | "zeph" | "zep" | "zp" => "Zephaniah",
            "haggai" | "hag" | "hg" => "Haggai",
            "zecharaih" | "zech" | "zec" | "zc" => "Zechariah",
            "matthew" | "matt" | "mt" => "Matthew",
            "mark" | "mrk" | "mar" | "mk" | "mr" => "Mark",
            "luke" | "luk" | "lk" => "Luke",
            "john" | "joh" | "jhn" | "jn" => "John",
            "acts" | "act" | "ac" => "Acts",
            "romans" | "rom" | "ro" | "rm" => "Romans",
            "1 corinthians" | "1 cor" | "1 co" | "i cor" | "i co" | "1cor" | "1co"
            | "i corinthians" | "1corinthians" | "1st corinthians" | "first corinthians" => {
                "1 Corinthians"
            }
            "2 corinthians" | "2 cor" | "2 co" | "ii cor" | "ii co" | "2cor" | "2co"
            | "ii corinthians" | "2corinthians" | "2nd corinthians" | "second corinthians" => {
                "2 Corinthians"
            }
            "galatians" | "gal" | "ga" => "Galatians",
            "ephesians" | "eph" | "ephes" => "Ephesians",
            "philippians" | "phil" | "php" | "pp" => "Philippians",
            "colossians" | "col" => "Colossians",
            "1 thessalonians"
            | "1 thess"
            | "1 thes"
            | "1 th"
            | "i thess"
            | "i thes"
            | "i th"
            | "1thessalonians"
            | "1thess"
            | "1thes"
            | "1th"
            | "1st thessalonians"
            | "first thessalonians" => "1 Thessalonians",
            "2 thessalonians"
            | "2 thess"
            | "2 thes"
            | "2 th"
            | "ii thess"
            | "ii thes"
            | "ii th"
            | "2thessalonians"
            | "2thess"
            | "2thes"
            | "2th"
            | "2nd thessalonians"
            | "second thessalonians" => "2 Thessalonians",
            "1 timothy" | "1 tim" | "1 ti" | "i tim" | "i ti" | "1timothy" | "1tim" | "1ti"
            | "1st timothy" | "first timothy" => "1 Timothy",
            "2 timothy" | "2 tim" | "2 ti" | "ii tim" | "ii ti" | "2timothy" | "2tim" | "2ti"
            | "2nd timothy" | "second timothy" => "2 Timothy",
            "titus" | "tit" | "ti" => "Titus",
            "Pphilemon" | "philem" | "phm" | "pm" => "Philemon",
            "hebrews" | "heb" => "Hebrews",
            "james" | "jas" | "jm" => "James",
            "1 peter" | "1 pet" | "1 pe" | "1 pt" | "1 p" | "i pet" | "i pe" | "i pt"
            | "1peter" | "1pet" | "1pe" | "1pt" | "1p" | "1st peter" | "first peter" => "1 Peter",
            "2 pet" | "2 pe" | "2 pt" | "2 p" | "ii pet" | "ii pe" | "ii pt" | "2peter"
            | "2pet" | "2pe" | "2pt" | "2p" | "2nd peter" | "second peter" => "2 Peter",
            "1 john" | "1 jhn" | "1 jn" | "1 j" | "1john" | "1jhn" | "1joh" | "1jn" | "1jo"
            | "1j" | "i john" | "i jhn" | "i joh" | "i jn" | "i jo" | "1st john" | "first john" => {
                "1 John"
            }
            "2 john" | "2 jhn" | "2 jn" | "2 j" | "2john" | "2jhn" | "2joh" | "2jn" | "2jo"
            | "2j" | "ii john" | "ii jhn" | "ii joh" | "ii jn" | "ii jo" | "2nd john"
            | "second john" => "2 John",
            "3 john" | "3 jhn" | "3 jn" | "3 j" | "3john" | "3jhn" | "3joh" | "3jn" | "3jo"
            | "3j" | "iii john" | "iii jhn" | "iii joh" | "iii jn" | "iii jo" | "3rd john"
            | "third john" => "3 John",
            "jude" | "jud" | "jd" => "Jude",
            "Revelation" | "rev" | "re" => "Revelation",
            // Default case
            _ => "Unknown Book",
        }
    }
}
