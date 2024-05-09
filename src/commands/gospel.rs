use crate::helpers::Config;

pub fn gospel(display_name: &str) -> Option<String> {
    let mut config = Config::load(&display_name);
    config.add_account_metrics_gospel_english();
    let command_success= "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"".to_string();
    Some(format!("{}", command_success))
}

pub fn evangelio(display_name: &str) -> Option<String> {
    let mut config = Config::load(&display_name);
    config.add_account_metrics_gospel_spanish();
    let command_success=  "El evangelio significa buenas nuevas! La mala noticia es que todos hemos pecado y merecemos la ira venidera. Pero Jesus, el Mesias, murio por nuestros pecados, fue sepultado y resucito al tercer dia segun las Escrituras. Ascendio a los cielos y esta sentado a la diestra del Padre. Jesus dijo: \"Yo soy el camino, la verdad y la vida. Nadie viene al Padre sino por mi. El tiempo se ha cumplido, y el reino de Dios se ha acercado; arrepentios y creed en el evangelio\".".to_string();
    Some(format!("{}", command_success))
}
pub fn evangelium(display_name: &str) -> Option<String> {
    let mut config = Config::load(&display_name);
    config.add_account_metrics_gospel_german();
    let command_success= "Evangelium bedeutet Gute Nachricht! Die schlechte Nachricht ist, wir haben alle gesundigt und verdienen Gottes Zorn. Doch Jesus Christus starb fur unsere Sunden, wurde begraben und am dritten Tag auferweckt, nach der Bibel. Er fuhr in den Himmel auf und sitzt jetzt zur Rechten des Vaters. Jesus sagt: \"Ich bin der Weg, die Wahrheit und das Leben; niemand kommt zum Vater ausser durch mich.\" Die Zeit ist reif und das Reich Gottes ist nahe; kehrt um und glaubt an das Evangelium.".to_string();
    Some(format!("{}", command_success))
}
