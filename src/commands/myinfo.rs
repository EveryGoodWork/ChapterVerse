use crate::helpers::Config;

const HELP_MESSAGE: &str = "MyInfo Help: Responds with the aggregated information and metrics stored about you. You can optionally specify to delete your information with, which will result in ChapterVerse leaving your channel on next restart.  Use !leavechannel for it to be immediate or !joinchannel to have it rejoin your channel as if a new user. Usage: !myinfo | !myinfo delete";
const UNRECOGNIZED_PARAMETER_MESSAGE: &str = "Unrecognized parameter, for help use: !myinfo ?";
const DELETE_SUCCESS_MESSAGE: &str = "Your information has been deleted.  Please note any use of ChapterVerse will create a new user record.";

pub async fn myinfo(display_name: &str, params: Vec<String>) -> Option<String> {    

    params.get(0).map_or_else(
        || {
            let config = Config::load(&display_name);
            
            let binding = String::new();
            let username = config
                .account
                .as_ref()
                .and_then(|acc| acc.username.as_ref())
                .unwrap_or(&binding);
            let translation = config
                .account
                .as_ref()
                .and_then(|acc| {
                    acc.bible
                        .as_ref()
                        .and_then(|bbl| bbl.preferred_translation.as_ref())
                })
                .unwrap_or(&binding);
            let date_added = config
                .account
                .as_ref()
                .and_then(|acc| acc.created_date.as_ref())
                .map(|dt| dt.format("%b %d, %Y").to_string())
                .unwrap_or_default();
            let last_verse = config
                .account
                .as_ref()
                .and_then(|acc| acc.bible.as_ref().and_then(|bbl| bbl.last_verse.as_ref()))
                .unwrap_or(&binding);
            let total_scriptures = config
                .account
                .as_ref()
                .and_then(|acc| {
                    acc.metrics
                        .as_ref()
                        .and_then(|mtr| mtr.scriptures.map(|s| s.to_string()))
                })
                .unwrap_or_default();
            let last_updated = config
                .account
                .as_ref()
                .and_then(|acc| acc.modified_date.as_ref())
                .map(|dt| dt.format("%b %d, %Y, %I:%M:%S %p").to_string())
                .unwrap_or_default();
    
            let total_gospels = config
                .account
                .as_ref()
                .and_then(|acc| {
                    acc.metrics.as_ref().map(|mtr| {
                        mtr.gospels_english.unwrap_or(0)
                            + mtr.gospels_spanish.unwrap_or(0)
                            + mtr.gospels_german.unwrap_or(0)
                    })
                })
                .unwrap_or(0);
    
            let channel_name = config
                .channel
                .as_ref()
                .and_then(|chn| chn.from_channel.as_ref())
                .unwrap_or(&binding);
            let joined_channel = config
                .channel
                .as_ref()
                .and_then(|chn| chn.active)
                .unwrap_or(false);
            let join_date = config
                .channel
                .as_ref()
                .and_then(|chn| chn.join_date.as_ref())
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or(String::from("Not joined"));
            

            Some(        format!("Username: {} | Translation: {} | DateAdded: {} | Last Scripture: {} | Total Scriptures: {} | Gospels Total: {} | Channel: {} | Joined: {} | Join Date: {} | Updated: {}", 
            username, translation, date_added, last_verse, total_scriptures, total_gospels, channel_name, joined_channel, join_date, last_updated)
)
        },
        |p| {
            let param = p.to_lowercase();
            if param == "?" || param == "help" {
                Some(HELP_MESSAGE.to_string())
            } else if param == "delete" || param == "del" {
                let config = Config::load(&display_name);
                config.delete();
                Some(DELETE_SUCCESS_MESSAGE.to_string())
            } else {
                Some(UNRECOGNIZED_PARAMETER_MESSAGE.to_string())
            }
        },
    )
}
