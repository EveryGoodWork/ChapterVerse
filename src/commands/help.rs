pub fn help(available_bibles: fn() -> String) -> Option<String> {
    let command_success = "Help: Lookup scripture by typing: gen 1:1 or 2 tim 3:16-17 nkjv. Commands: !joinchannel, !translation, !votd, !next, !random, !previous, !leavechannel, !myinfo, !channelinfo, !support, !status, !setcommandprefix, !setvotd, !gospel, !evangelio, !evangelium, gospel message. Available translations:";
    Some(format!("{} {}", command_success, available_bibles()).to_string())
}
