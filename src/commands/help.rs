pub fn help(available_bibles: fn() -> String, prefix: &char) -> Option<String> {
    let command_success = "Help: Lookup scripture by typing: gen 1:1 or 2 tim 3:16-17 nkjv. Commands: !joinchannel, !translation, !votd, !next, !random, !previous, !leavechannel, !myinfo, !channelinfo, !support, !status, !commandprefix, !setvotd, !gospel, !evangelio, !evangelium, gospel message. Available translations:";
    let command_success = command_success.replace("!", &prefix.to_string());
    Some(format!("{} {}", command_success, available_bibles()).to_string())
}
