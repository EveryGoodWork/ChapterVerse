use crate::helpers::Config;

pub async fn myinfo(display_name: &str, params: Vec<String>) -> Option<String> {
    let help_message = "MyInfo Help: Responds with the aggregated information and metrics stored about you. You can optionally specify to delete your information with, which will result in ChapterVerse leaving your channel on next restart.  Use !leavechannel for it to be immediate or !joinchannel to have it rejoin your channel as if a new user. Usage: !myinfo | !myinfo delete";

    params.get(0).map_or_else(
        || {
            let config = Config::load(&display_name);
            Some(config.get_details())
        },
        |p| {
            let param = p.to_lowercase();
            if param == "?" || param == "help" {
                Some(help_message.to_string())
            } else if param == "delete" || param == "del" {
                let config = Config::load(&display_name);
                config.delete();
                Some("Your information has been deleted.".to_string())
            } else {
                let config = Config::load(&display_name);
                Some(config.get_details())
            }
        },
    )
}
