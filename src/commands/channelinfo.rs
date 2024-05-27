use crate::helpers::Config;

const HELP_MESSAGE: &str = "ChannelInfo Help: Responds with the aggregated information and metrics stored about the channel. 
The channel owner can optionally specify to delete their information, which will result in ChapterVerse leaving your channel on next restart.  
Use !leavechannel for it to be immediate or !joinchannel to have it rejoin your channel as if a new user. Usage: !channelinfo | !channelinfo delete";
const UNRECOGNIZED_PARAMETER_MESSAGE: &str = "Unrecognized parameter, for help use: !channelinfo ?";
const DELETE_SUCCESS_MESSAGE: &str = "information on has been deleted, and will leave the channel on next restart.  To leave channel immediately use: !leavechannel";
const DELETE_DENIED_MESSAGE: &str = "Only the channel owner can delete their channel's info.";

pub async fn channelinfo(channel: &str, display_name: &str, params: Vec<String>) -> Option<String> {
    params.get(0).map_or_else(
        || {
            let config = Config::load(&channel);
            let empty = String::new();

            let last_updated = config
            .channel
            .as_ref()
            .and_then(|cha| cha.modified_date.as_ref())
            .map_or_else(String::new, |dt| {
                dt.format("%b %d, %Y, %H:%M:%S UTC").to_string()
            });

            let join_date = config
                .channel
                .as_ref()
                .and_then(|chn| chn.join_date.as_ref())
                .map_or_else(
                    || String::from("Not joined"),
                    |dt| dt.format("%Y-%m-%d").to_string(),
                );

            let total_scriptures = config
                .channel
                .as_ref()
                .and_then(|cha| cha.metrics.as_ref())
                .and_then(|mtr| mtr.scriptures.map(|s| s.to_string()))
                .unwrap_or_default();

            let total_gospels = config
                .channel
                .as_ref()
                .and_then(|cha| cha.metrics.as_ref())
                .map_or(0, |mtr| {
                    mtr.gospels_english.unwrap_or(0)
                        + mtr.gospels_spanish.unwrap_or(0)
                        + mtr.gospels_german.unwrap_or(0)
                });

            let joined_from_channel = config
                .channel
                .as_ref()
                .and_then(|channel| channel.from_channel.as_ref())
                .unwrap_or(&empty);

            Some(format!(
                "Channel: {} | Total Scriptures: {} | Total Gospels: {} | Joined From Channel: {} | Date Joined: {} | Last Updated: {}",
                channel, total_scriptures, total_gospels, joined_from_channel, join_date, last_updated))
        },
        |p| {
            let param = p.to_lowercase();
            if param == "?" || param == "help" {
                Some(HELP_MESSAGE.to_string())
            } else if param == "delete" {
                if channel.eq_ignore_ascii_case(display_name) {
                    let config = Config::load(&display_name);
                    config.delete();
                    Some(format!("{} {}",channel, DELETE_SUCCESS_MESSAGE))
                } else {
                    Some(DELETE_DENIED_MESSAGE.to_string())
                }
            } else {
                Some(UNRECOGNIZED_PARAMETER_MESSAGE.to_string())
            }
        },
    )
}
