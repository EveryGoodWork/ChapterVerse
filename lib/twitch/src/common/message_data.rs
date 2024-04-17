use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    PossibleCommand,
    PossibleScripture,
    NotScripture,
    NotCommand,
    Command,
    Scripture,
    Gospel,
    None,
    Ignore,
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub badge_info: Option<&'static str>,
    pub badges: Option<&'static str>,
    pub client_nonce: Option<&'static str>,
    pub color: Option<&'static str>,
    pub display_name: Option<&'static str>,
    pub emotes: Option<&'static str>,
    pub first_msg: Option<bool>,
    pub flags: Option<&'static str>,
    pub id: Option<&'static str>,
    pub mod_status: Option<bool>,
    pub returning_chatter: Option<bool>,
    pub room_id: Option<&'static str>,
    pub subscriber: Option<bool>,
    pub tmi_sent_ts: Option<&'static str>,
    pub turbo: Option<bool>,
    pub user_id: Option<&'static str>,
    pub user_type: Option<&'static str>,
    pub channel: String,
    pub text: String,
    pub raw_message: String,
    pub received: SystemTime,
    pub tags: Vec<Type>,
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
            tags: vec![],
            reply: None,
        }
    }

    // TODO!  Determine if this is the best place to do this, as technically I do this again in main.rs
    pub fn determine_message_types(text: &str) -> Vec<Type> {
        let mut types = Vec::new();
        let text_to_lowercase = text.to_lowercase();

        if text_to_lowercase.starts_with("!") {
            types.push(Type::PossibleCommand);
        } else if text.to_lowercase().contains("gospel message") {
            types.push(Type::Gospel);
        } else if text_to_lowercase.contains(":") {
            types.push(Type::PossibleScripture);
        } else if types.is_empty() {
            types.push(Type::None);
        }
        types
    }

    pub fn complete(&self) -> Result<u128, &'static str> {
        SystemTime::now()
            .duration_since(self.received)
            .map_err(|_| "Time went backwards")
            .map(|dur| dur.as_millis()) // Convert Duration to milliseconds
    }

    pub fn new(raw: &str) -> Option<Self> {
        //println!("*parse_message: {}", raw);

        let meta_content_split = raw.split_once(":")?;
        let (meta, account_and_message) = meta_content_split;
        let content_split = account_and_message.split_once("PRIVMSG #")?;
        let (channel, text) = if raw.starts_with("PRIVMSG #") {
            let parts: Vec<&str> = raw.splitn(3, ' ').collect();
            if parts.len() == 3 {
                (
                    parts[1].trim_start_matches('#').to_string(),
                    parts[2].trim_start_matches(':').to_string(),
                )
            } else {
                return None;
            }
        } else {
            let message = content_split.1.split_once(" :")?;
            (
                message.0.to_string(),
                message.1.trim_end_matches("\r\n").to_string(),
            )
        };

        let text_for_types = text.clone();
        let mut message = MessageData {
            text,
            raw_message: raw.to_string(),
            channel,
            tags: Self::determine_message_types(&text_for_types),
            ..Self::default()
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
        // TODO! Pull these names from a configuration option.
        let accounts_to_ignore = ["EveryGoodWork", "ChapterVerse"];
        if let Some(display_name) = message.display_name {
            if accounts_to_ignore.iter().any(|&name| name == display_name) {
                message.tags.push(Type::Ignore);
                return Some(message);
            }
        }
        Some(message)
    }
}
