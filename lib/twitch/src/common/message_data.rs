use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Command,
    PossibleScripture,
    Scripture,
    Gospel,
    None,
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub badge_info: Option<&'static str>,
    pub badges: Option<&'static str>,
    pub client_nonce: Option<&'static str>,
    pub color: Option<&'static str>,
    pub display_name: Option<&'static str>,
    pub emotes: Option<&'static str>,
    pub first_msg: Option<u8>,
    pub flags: Option<&'static str>,
    pub id: Option<&'static str>,
    pub mod_status: Option<u8>,
    pub returning_chatter: Option<u8>,
    pub room_id: Option<&'static str>,
    pub subscriber: Option<u8>,
    pub tmi_sent_ts: Option<&'static str>,
    pub turbo: Option<u8>,
    pub user_id: Option<&'static str>,
    pub user_type: Option<&'static str>,
    pub channel: String,
    pub text: String,
    pub raw_message: String,
    pub received: SystemTime,
    pub classification: Type,
    pub reply: Option<String>,
}

impl MessageData {
    pub fn default() -> Self {
        MessageData {
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
            channel: String::new(),
            text: String::new(),
            raw_message: String::new(),
            received: SystemTime::now(),
            classification: Type::None,
            reply: None,
        }
    }

    pub fn complete(&self) -> Result<Duration, &'static str> {
        match SystemTime::now().duration_since(self.received) {
            Ok(duration) => {
                // println!("Message processing duration: {:?}", duration);
                Ok(duration)
            }
            Err(_) => Err("Time went backwards"),
        }
    }

    pub fn new(text: &str, raw_message: &str) -> Self {
        let message_type = MessageData::determine_message_type(text);

        MessageData {
            text: text.to_string(),
            raw_message: raw_message.to_string(),
            classification: message_type,
            ..MessageData::default()
        }
    }
    pub fn determine_message_type(text: &str) -> Type {
        if text.starts_with("!") {
            Type::Command
        } else if text.contains(":") {
            Type::PossibleScripture
        } else {
            Type::None
        }
    }
}

pub fn parse_message(message_text: &str) -> Option<MessageData> {
    println!("parse_message: {}", message_text);

    if message_text.starts_with("PRIVMSG #") {
        let parts: Vec<&str> = message_text.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let channel = parts[1].trim_start_matches('#');
            let text = parts[2].trim_start_matches(':');
            let message_type = if text.starts_with("!") {
                Type::Command
            } else if text.contains(":") {
                Type::PossibleScripture
            } else {
                Type::None
            };
            return Some(MessageData {
                channel: channel.to_string(),
                text: text.to_string(),
                raw_message: message_text.to_string(),
                classification: message_type,
                reply: None,
                ..MessageData::default()
            });
        }
        return None;
    }

    let meta_content_split = message_text.split_once(":")?;
    let (meta, account_and_message) = meta_content_split;
    let content_split = account_and_message.split_once("PRIVMSG #")?;
    let message = content_split.1.split_once(" :")?;
    let (channel, text) = (
        message.0.to_string(),
        message.1.trim_end_matches("\r\n").to_string(),
    );
    let message_type = MessageData::determine_message_type(message_text);

    let mut message = MessageData {
        channel,
        text,
        raw_message: message_text.to_string(),
        classification: message_type,
        reply: None,
        ..MessageData::default()
    };

    for part in meta.split(";") {
        if let Some((key, value)) = part.split_once("=") {
            let value_static = Box::leak(value.to_string().into_boxed_str());
            match key {
                "badge-info" => message.badge_info = Some(value_static),
                "badges" => message.badges = Some(value_static),
                "client-nonce" => message.client_nonce = Some(value_static),
                "color" => message.color = Some(value_static),
                "display-name" => message.display_name = Some(value_static),
                "emotes" => message.emotes = Some(value_static),
                "first-msg" => message.first_msg = value.parse().ok(),
                "flags" => message.flags = Some(value_static),
                "id" => message.id = Some(value_static),
                "mod" => message.mod_status = value.parse().ok(),
                "returning-chatter" => message.returning_chatter = value.parse().ok(),
                "room-id" => message.room_id = Some(value_static),
                "subscriber" => message.subscriber = value.parse().ok(),
                "tmi-sent-ts" => message.tmi_sent_ts = Some(value_static),
                "turbo" => message.turbo = value.parse().ok(),
                "user-id" => message.user_id = Some(value_static),
                "user-type" => message.user_type = Some(value_static),
                _ => {}
            }
        }
    }

    Some(message)
}
