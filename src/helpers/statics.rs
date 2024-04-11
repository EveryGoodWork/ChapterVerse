use crate::helpers::env_variables::get_env_variable;
use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};

lazy_static! {

pub static ref GOSPEL: String = "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"".to_string();
pub static ref EVANGELIO: String = "El evangelio significa buenas nuevas! La mala noticia es que todos hemos pecado y merecemos la ira venidera. Pero Jesus, el Mesias, murio por nuestros pecados, fue sepultado y resucito al tercer dia segun las Escrituras. Ascendio a los cielos y esta sentado a la diestra del Padre. Jesus dijo: \"Yo soy el camino, la verdad y la vida. Nadie viene al Padre sino por mi. El tiempo se ha cumplido, y el reino de Dios se ha acercado; arrepentios y creed en el evangelio\".".to_string();
pub static ref EVANGELIUM: String = "Evangelium bedeutet Gute Nachricht! Die schlechte Nachricht ist, wir haben alle gesundigt und verdienen Gottes Zorn. Doch Jesus Christus starb fur unsere Sunden, wurde begraben und am dritten Tag auferweckt, nach der Bibel. Er fuhr in den Himmel auf und sitzt jetzt zur Rechten des Vaters. Jesus sagt: \"Ich bin der Weg, die Wahrheit und das Leben; niemand kommt zum Vater ausser durch mich.\" Die Zeit ist reif und das Reich Gottes ist nahe; kehrt um und glaubt an das Evangelium.".to_string();
    // TODO!  Pull this from file.
pub static ref CHANNELS_TO_JOIN: Vec<String> = {
    vec![
        "chapterverse".to_string(),
        "missionarygamer".to_string(),
        "kcchurch".to_string(),
        "madhelp".to_string(),
        "streamintel".to_string(),
        "carol_ai".to_string(),
        "madmeshes".to_string(),
        "linuxmountian".to_string(),
        "host_ai".to_string(),
        "twitchstreamintel".to_string(),
        "FrankLayman".to_string(),
        "Husky_Pup".to_string(),
        "Spurgeon_AI".to_string(),
        "Bob_AI".to_string(),
        "ScatteredWisdom".to_string(),
        "biblicalwisdom".to_string(),
        "overlaygames_bot".to_string(),
        "Guest_AI".to_string(),
        "blipzak".to_string(),
        "TodAIshow".to_string(),
        "fireresistant".to_string(),
    ]
};
pub static ref BIBLES: Arc<HashMap<String, Arc<Bible>>> = {

            let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");

            let bibles_directory = match env::current_dir().map(|dir| dir.join(import_bibles_path)) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("Error getting current directory: {}", e);
                    return Arc::new(HashMap::new());
                }
            };

            let mut bibles = HashMap::new();

            let files = match fs::read_dir(bibles_directory) {
                Ok(files) => files,
                Err(e) => {
                    println!("Error reading bibles directory: {}", e);
                    return Arc::new(HashMap::new());
                }
            };

            for file in files {
                let entry = match file {
                    Ok(entry) => entry,
                    Err(e) => {
                        println!("Error reading file in directory: {}", e);
                        continue; // Skip to the next iteration
                    }
                };

                if entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("csv")
                {
                    let file_stem = entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string()
                        .to_uppercase();
                    let file_path = entry.path().to_string_lossy().to_string();
                    match bible_import(&entry.path().to_string_lossy()) {
                        Ok(imported_bible) => {
                            bibles.insert(file_stem, Arc::new(imported_bible));
                        }
                        Err(err) => {
                            println!("Error running import for file '{}': {}", file_path, err);
                        }
                    }
                }
            }

            Arc::new(bibles)
        };
    }
#[allow(unused)]
fn get_bibles_names() -> String {
    BIBLES.keys().cloned().collect::<Vec<_>>().join(", ")
}
#[allow(unused)]
fn get_specific_bible(bible_name: &str) -> Option<Arc<Bible>> {
    let bibles = Arc::clone(&BIBLES); // Clone the Arc for thread-safe access
    let lookup_name = bible_name.to_uppercase(); // Convert the lookup name to lowercase
    bibles.get(&lookup_name).cloned()
}
