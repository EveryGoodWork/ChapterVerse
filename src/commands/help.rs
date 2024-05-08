pub fn help(available_bibles: fn() -> String) -> Option<String> {
    Some(format!(
        "HELP: Available translations: {}. Lookup by typing: gen 1:1 or 2 tim 3:16-17 niv. Commands: !help, !joinchannel, !votd, !random, !next, !previous, !leavechannel, !myinfo, !channelinfo, !support, !status, !setcommandprefix, !setvotd, !gospel, !evangelio, !evangelium, gospel message.",
        available_bibles()
    ).to_string())
}
