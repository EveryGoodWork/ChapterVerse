#[derive(Debug)]
pub struct MessageData {
    pub badge_info: Option<String>,
    pub badges: Option<String>,
    pub client_nonce: Option<String>,
    pub color: Option<String>,
    pub display_name: Option<String>,
    pub emotes: Option<String>,
    pub first_msg: Option<u8>,
    pub flags: Option<String>,
    pub id: Option<String>,
    pub mod_status: Option<u8>,
    pub returning_chatter: Option<u8>,
    pub room_id: Option<String>,
    pub subscriber: Option<u8>,
    pub tmi_sent_ts: Option<String>,
    pub turbo: Option<u8>,
    pub user_id: Option<String>,
    pub user_type: Option<String>,
    pub channel: String,
    pub text: String,
}

pub fn parse_message(raw_message: &str) -> Option<MessageData> {
    let meta_content_split = raw_message.split_once(" :")?;
    let (meta, content) = meta_content_split;

    // Splitting to extract channel and text
    let content_split = content.split_once("PRIVMSG #")?;
    let channel_text_split = content_split.1.split_once(" :")?;
    let (channel, text) = (
        channel_text_split.0.to_string(),
        channel_text_split.1.to_string(),
    );

    let mut message = MessageData {
        badge_info: None,
        badges: None,
        client_nonce: None,
        color: None,
        display_name: None,
        emotes: None,
        first_msg: None,
        flags: None,
        id: None,
        mod_status: None,
        returning_chatter: None,
        room_id: None,
        subscriber: None,
        tmi_sent_ts: None,
        turbo: None,
        user_id: None,
        user_type: None,
        channel,
        text,
    };

    for part in meta.split(";") {
        if let Some((key, value)) = part.split_once("=") {
            match key {
                "badge-info" => message.badge_info = Some(value.to_string()),
                "badges" => message.badges = Some(value.to_string()),
                "client-nonce" => message.client_nonce = Some(value.to_string()),
                "color" => message.color = Some(value.to_string()),
                "display-name" => message.display_name = Some(value.to_string()),
                "emotes" => message.emotes = Some(value.to_string()),
                "first-msg" => message.first_msg = value.parse().ok(),
                "flags" => message.flags = Some(value.to_string()),
                "id" => message.id = Some(value.to_string()),
                "mod" => message.mod_status = value.parse().ok(),
                "returning-chatter" => message.returning_chatter = value.parse().ok(),
                "room-id" => message.room_id = Some(value.to_string()),
                "subscriber" => message.subscriber = value.parse().ok(),
                "tmi-sent-ts" => message.tmi_sent_ts = Some(value.to_string()),
                "turbo" => message.turbo = value.parse().ok(),
                "user-id" => message.user_id = Some(value.to_string()),
                "user-type" => message.user_type = Some(value.to_string()),
                _ => {} // Ignore unknown keys or add to a HashMap if needed
            }
        }
    }

    Some(message)
}
