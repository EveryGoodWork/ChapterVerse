use bible::scripture::bible::Bible;
use helpers::print_color::PrintCommand;
use tokio::sync::mpsc;

use futures::future::pending;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use twitch::chat::client::WebSocketState;
use twitch::chat::Listener;
use twitch::chat::Replier;
use twitch::common::message_data::{MessageData, Type};

use helpers::config::Config;
use helpers::env_variables::get_env_variable;
use helpers::statics::{avaialble_bibles, find_bible};
use helpers::statics::{BIBLES, CHANNELS_TO_JOIN, EVANGELIO, EVANGELIUM, GOSPEL};

mod helpers;

const CHANNELS_PER_LISTENER: usize = 5;

#[tokio::main]
async fn main() {
    PrintCommand::System.print_message("ChapterVerse", "Jesus is Lord!");
    PrintCommand::Issue.print_message("Version", env!("CARGO_PKG_VERSION"));
    PrintCommand::Info.print_message("What is the Gospel?", "Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel.\"");
    for (bible_name, bible_arc) in BIBLES.iter() {
        let bible: &Bible = &*bible_arc; // Dereference the Arc and immediately borrow the result
        let scripture = match bible.get_scripture("2 Timothy 3:16") {
            Some(verse) => format!("{}", verse.scripture),
            None => "Verse not found".to_string(),
        };
        PrintCommand::Info.print_message(&format!("{}, 2 Timothy 3:16", bible_name), &scripture);
    }

    let (listener_transmitter, mut listener_reciever) = mpsc::unbounded_channel::<MessageData>();
    let (replier_transmitter, mut replier_receiver) = mpsc::unbounded_channel::<MessageData>();

    let listeners = Arc::new(Mutex::new(HashMap::<String, Arc<Listener>>::new()));

    let twitch_account = get_env_variable("TWITCHACCOUNT", "twitchusername");
    let twitch_oauth = get_env_variable("TWITCHOAUTH", "oauth:1234p1234p1234p1234p1234p1234p");
    let replier = Arc::new(Replier::new(&twitch_account, &twitch_oauth));

    let replier_transmitter_clone = Arc::new(Listener::new(replier_transmitter.clone()));
    let listeners_clone = Arc::clone(&listeners);
    let listener_transmitter_clone = listener_transmitter.clone();
    // Spawn a task to Listens for incoming Twitch messages.
    tokio::spawn(async move {
        while let Some(mut message) = listener_reciever.recv().await {
            let mut reply: Option<String> = None;
            let display_name = message.display_name.unwrap();
            let tags = message.tags.clone();

            if !tags.contains(&Type::Ignore) {
                for tag in tags {
                    match tag {
                        Type::None => (),
                        Type::Gospel => reply = Some(GOSPEL.to_string()),
                        Type::PossibleCommand => {
                            let input = &message.text.as_str().to_lowercase();
                            let mut parts = input.split_whitespace();
                            let command = parts.next().unwrap_or_default().to_string();
                            let params: Vec<String> = parts.map(|s| s.to_string()).collect();

                            reply = match command.as_str() {
                                // TODO!  Get the list of avaialble translations dynamically.
                                "!help" => {
                                    message.tags.push(Type::Command);
                                    Some(format!("HELP: Available translations: {}. Lookup by typing: gen 1:1 or 2 tim 3:16-17 niv. Commands: !help, !joinchannel, !votd, !random, !next, !previous, !leavechannel, !myinfo, !channelinfo, !support, !status, !setcommandprefix, !setvotd, !gospel, !evangelio, !evangelium, gospel message.", avaialble_bibles()).to_string())
                                }
                                "!joinchannel" => {
                                    // TODO!  Handle not joining channels that have already been joined, as this results in two listeners being attached to the channel.
                                    message.tags.push(Type::Command);
                                    let mut config = Config::load(&display_name);
                                    if !config.channel.as_ref().unwrap().date_joined.is_some() {
                                        config.join_channel();
                                        let new_twitch_listener = Arc::new(Listener::new(
                                            listener_transmitter_clone.clone(),
                                        ));
                                        match new_twitch_listener.clone().connect().await {
                                            Ok(_) => {
                                                println!("Successfully connected. - Not Actually - it is in process");
                                                let _ = new_twitch_listener
                                                    .clone()
                                                    .join_channel(display_name)
                                                    .await;
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to connect: {:?}", e);
                                                tokio::time::sleep(
                                                    tokio::time::Duration::from_secs(5),
                                                )
                                                .await;
                                                continue;
                                            }
                                        }
                                        let listeners_lock = listeners_clone.lock();
                                        listeners_lock
                                            .await
                                            .insert(display_name.to_string(), new_twitch_listener);
                                        Some(
                                            format!(
                                                "Joined channel {}",
                                                message.display_name.unwrap_or_default()
                                            )
                                            .to_string(),
                                        )
                                    } else {
                                        Some(
                                            format!(
                                                "Already joined {} on : {}",
                                                message.display_name.unwrap_or_default(),
                                                config
                                                    .channel
                                                    .unwrap()
                                                    .date_joined
                                                    .unwrap_or_default()
                                            )
                                            .to_string(),
                                        )
                                    }
                                }
                                "!translation" => {
                                    message.tags.push(Type::Command);
                                    let mut config = Config::load(&display_name);
                                    let translation = params[0].to_uppercase();

                                    if BIBLES.contains_key(&translation) {
                                        config.preferred_translation(&translation);
                                        config.add_note(
                                            format!("!translation {}", &translation).to_owned(),
                                        );
                                        Some(
                                            format!(
                                                "Set perferred translation: {}.",
                                                config.get_translation()
                                            )
                                            .to_string(),
                                        )
                                    } else {
                                        Some(
                                            format!(
                                                "Available translations: {}.",
                                                avaialble_bibles()
                                            )
                                            .to_string(),
                                        )
                                    }
                                }
                                "!votd" => Some("Display the verse of the day.".to_string()),
                                "!random" => Some("Display a random verse.".to_string()),
                                "!next" => Some("Go to the next item.".to_string()),
                                "!previous" => Some("Go to the previous item.".to_string()),
                                "!leavechannel" => Some("Leave the current channel.".to_string()),
                                "!myinfo" => Some("Display user's information.".to_string()),
                                "!support" => Some("Display support options.".to_string()),
                                "!status" => Some("Display current status.".to_string()),
                                "!setcommandprefix" => Some("Set the command prefix.".to_string()),
                                "!setvotd" => Some("Set the verse of the day.".to_string()),
                                "!gospel" => {
                                    message.tags.push(Type::Gospel);
                                    Some(GOSPEL.to_string())
                                }
                                "!evangelio" => {
                                    message.tags.push(Type::Gospel);
                                    Some(EVANGELIO.to_string())
                                }
                                "!evangelium" => {
                                    message.tags.push(Type::Gospel);
                                    Some(EVANGELIUM.to_string())
                                }
                                _ => {
                                    message.tags.push(Type::NotCommand);
                                    None
                                }
                            };
                        }
                        Type::PossibleScripture => {
                            let mut config = Config::load(&display_name);

                            let bible_name_to_use =
                                find_bible(&message.text, &config.get_translation());
                            config.last_translation(&bible_name_to_use);

                            if let Some(bible_arc) = BIBLES.get(&bible_name_to_use) {
                                let bible: &Bible = &*bible_arc;
                                reply = match bible.get_scripture(&message.text) {
                                    Some(verse) => {
                                        message.tags.push(Type::Scripture);
                                        let scripture = format!(
                                            "{} - {} {}",
                                            verse.scripture, verse.abbreviation, bible_name_to_use
                                        );
                                        config.last_verse(&verse.reference);
                                        Some(scripture)
                                    }
                                    None => {
                                        message.tags.push(Type::NotScripture);
                                        //Some("Verse not found".to_string())
                                        None
                                    }
                                };
                                PrintCommand::Info.print_message(
                                    &format!(
                                        "Bible {}, {:?}",
                                        bible_name_to_use, message.display_name
                                    ),
                                    format!("{:?}", reply).as_str(),
                                );
                            } else {
                                eprintln!("Bible named '{}' not found.", bible_name_to_use);
                            }
                        }
                        _ => {
                            {
                                //Handle other message types here if needed
                            }
                        }
                    }
                    match reply {
                        Some(ref reply_value) => {
                            // TODO!  This is where I'll put the configuration update.
                            println!("Tages: {:?}", message.tags);
                            message.reply = Some(format!(
                                "{} ({})",
                                reply_value,
                                message
                                    .complete()
                                    .ok()
                                    .map_or_else(|| "".to_string(), |d| format!("{:?}", d))
                            ));
                            if let Err(e) =
                                replier_transmitter_clone.message_tx.send(message.clone())
                            {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                        None => {
                            println!("NONE: {:?}", reply);
                            let _ = message.complete();
                        }
                    }
                }
            }
        }
    });

    let listeners_clone = Arc::clone(&listeners);
    let listener_transmitter_clone = listener_transmitter.clone();
    // Spawn a task to manage connections, listeners, and reconnection
    tokio::spawn(async move {
        loop {
            let new_twitch_listener = Arc::new(Listener::new(listener_transmitter_clone.clone()));
            // TODO! - this looks like I'm creating this for no reason now that I'm doing the loop.  This needs to be refactored.
            match new_twitch_listener.clone().connect().await {
                Ok(_) => println!("Websocket connect OK..."),
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            for chunk in CHANNELS_TO_JOIN.chunks(CHANNELS_PER_LISTENER) {
                let chuch_twitch_listener =
                    Arc::new(Listener::new(listener_transmitter_clone.clone()));
                let listeners_lock = listeners_clone.lock();
                listeners_lock.await.insert(
                    chuch_twitch_listener.username.to_string(),
                    chuch_twitch_listener.clone(),
                );
                match chuch_twitch_listener.clone().connect().await {
                    Ok(_) => println!("Successfully connected. - Not Actually - it is in process"),
                    Err(e) => {
                        eprintln!("Failed to connect: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
                tokio::spawn(async move {
                    for channel in chunk {
                        let twitch_listener_clone = Arc::clone(&chuch_twitch_listener); // Clone for each iteration
                        let username = twitch_listener_clone.username.to_string();
                        match twitch_listener_clone.join_channel(channel).await {
                            Ok(_) => println!("{} Joined channel {}", username, channel),
                            Err(e) => eprintln!("Failed to join channel {}: {}", channel, e),
                        }
                    }
                });
            }
            while new_twitch_listener.clone().get_state() != WebSocketState::Disconnected {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    // Spawn a task for replying to messages.
    let replier_clone = Arc::clone(&replier);
    let loop_replier_clone = Arc::clone(&replier);
    tokio::spawn(async move {
        match replier_clone.clone().connect().await {
            Ok(_) => {
                println!("Successfully connected for Replying.");
                let _ = replier_clone
                    .clone()
                    .send_message("chapterverse", "Jesus is Lord!")
                    .await;
                let _ = replier_clone
                    .clone()
                    .send_message(
                        "chapterverse",
                        format!(
                            "ChapterVerse Version: {} - ONLINE",
                            env!("CARGO_PKG_VERSION")
                        )
                        .as_str(),
                    )
                    .await;

                // // Test Loop to send 100 messages with a counter and the current time.
                // for count in 1..=10 {
                //     if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                //         let timestamp = now.as_secs(); // Seconds since UNIX epoch
                //         let message = format!("Debug Count: {} - Timestamp: {}", count, timestamp);
                //         let _ = replier_clone
                //             .clone()
                //             .send_message("chapterverse", &message)
                //             .await;
                //     }
                // }
            }
            Err(e) => {
                eprintln!("Failed to connect: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
        while let Some(message) = replier_receiver.recv().await {
            // TODO!  Find out about if I can remove these clones.
            let _ = loop_replier_clone
                .clone()
                // TODO! Update MessageData with a reply_text field
                .reply_message(message)
                .await;
        }
    });

    // This line will keep the program running indefinitely until it's killed manually (e.g., Ctrl+C).
    pending::<()>().await;
}

#[cfg(test)]
mod unittests {
    use super::*;
    // use the following command line to see the results of the test: cargo test -- --nocapture
    #[test]
    fn get_scripture() {
        for (bible_name, bible_arc) in BIBLES.iter() {
            let bible: &Bible = &*bible_arc; // Here you dereference the Arc and immediately borrow the result

            let message = match bible.get_scripture("2 Timothy 3:16") {
                Some(verse) => format!("{}", verse.scripture),
                None => "Verse not found".to_string(),
            };

            PrintCommand::Info.print_message(&format!("{}, 2 Timothy 3:16", bible_name), &message);
        }
    }
}
