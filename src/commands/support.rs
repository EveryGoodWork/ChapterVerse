pub fn support() -> Option<String> {
    let command_success = "
    ChapterVerse (https://github.com/EveryGoodWork/ChapterVerse) is open source and will always be free to use for the Twitch community. To support development costs for this and other faith-inspired software, please consider becoming a patron: https://www.patreon.com/missionarygamer.";
    Some(format!("{}", command_success))
}
