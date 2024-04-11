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
use twitch::common::message_data::MessageData;
use twitch::common::message_data::Type;

use crate::helpers::config::Config;
use crate::helpers::env_variables::get_env_variable;
use crate::helpers::statics::BIBLES;
use crate::helpers::statics::CHANNELS_TO_JOIN;
use crate::helpers::statics::{EVANGELIO, EVANGELIUM, GOSPEL};

mod helpers;
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
    let listeners = Arc::new(Mutex::new(HashMap::<String, Arc<Listener>>::new()));
    let twitch_listener = Arc::new(Listener::new(listener_transmitter.clone()));
    let (txreplier, rxreplier) = mpsc::unbounded_channel::<MessageData>();
    let mut rxreplier_clone = rxreplier;

    const CHANNELS_PER_LISTENER: usize = 5;
    // Spawn a taslk to Listens for incoming Twitch messages.

    tokio::spawn(async move {
        while let Some(mut message) = listener_reciever.recv().await {
            let mut reply: Option<String> = None;
            let tags = message.tags.clone();
            if !tags.contains(&Type::Ignore) {
                for tag in tags {
                    match tag {
                        Type::None => (),
                        Type::Gospel => reply = Some(GOSPEL.to_string()),
                        Type::Command => {
                            let command = message.text.as_str().to_lowercase();

                            reply = match command.as_str() {
                                // TODO!  Get the list of avaialble translations dynamically.
                                "!help" => Some(" Available translations: AMP, ESV (default), KJV, NASB, NIV, NKJV, Web. Lookup by typing: gen 1:1 or 2 tim 3:16-17 niv. Commands: !help, !joinchannel, !votd, !random, !next, !previous, !leavechannel, !myinfo, !channelinfo, !support, !status, !setcommandprefix, !setvotd, !gospel, !evangelio, !evangelium, gospel message.".to_string()),
                                "!joinchannel" =>{                                     
                                    println!("Join a channel {}", message.channel);
                                    let config: Config = Config::new();                                    
                                    println!("Channel name: {}", config.channel_name().unwrap_or_else(|e| e.to_string()));
                                    println!("Temp: {}", config.temp().unwrap_or_else(|e| e.to_string()));
                                    
                                    Some("Join a channel.".to_string())},
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
                                // TODO! This needs to be conbined into 1.
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
                                    // TODO!  Add the other versions of the Gospel.
                                    Some(EVANGELIUM.to_string())
                                }
                                _ => None,
                            };

                            // if let Some(response) = reply {
                            //     println!("Command: {}", response);
                            //     let completed = message.complete();
                            //     message.reply =
                            //         Some(format!("{} - {:?}", response.to_string(), completed));
                            //     if let Err(e) = txreplier.send(message.clone()) {
                            //         eprintln!("Failed to send cloned message: {}", e);
                            //     }
                            // } else {
                            //     println!("Unknown command.");
                            // }
                        }
                        Type::PossibleScripture => {
                            // TODO! Pull bible preference from env or context of the request.
                            let bible_name_to_use = "KJV";
                            if let Some(bible_arc) = BIBLES.get(bible_name_to_use) {
                                let bible: &Bible = &*bible_arc;
                                reply = match bible.get_scripture(&message.text) {
                                    Some(verse) => {
                                        message.tags.push(Type::Scripture);
                                        let scripture =
                                            format!("{} - {}", verse.scripture, verse.abbreviation);
                                        Some(scripture)
                                    }
                                    None => {
                                        message.tags.push(Type::NotScripture);
                                        Some("Verse not found".to_string())
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
                                //TODO! Handle other message types here if needed
                            }
                        }
                    }
                    match reply {
                        Some(ref reply_value) => {
                            message.reply = Some(format!(
                                "{} ({})",
                                reply_value,
                                message
                                    .complete()
                                    .ok()
                                    .map_or_else(|| "".to_string(), |d| format!("{:?}", d))
                            ));
                            if let Err(e) = txreplier.send(message.clone()) {
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
            // match message.complete() {
            //     Ok(duration) => println!(
            //         "Message processing duration: {:?}={:?}",
            //         message.tags, duration
            //     ),
            //     Err(e) => eprintln!("Error calculating duration: {}", e),
            // }
        }
    });

    // Spawn a task to manage connections, listeners, and reconnection
    tokio::spawn(async move {
        loop {
            let loop_listener_clone = Arc::clone(&twitch_listener);
            match loop_listener_clone.clone().connect().await {
                Ok(_) => println!("Successfully connected. - Not Actually - it is in process"),
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            for chunk in CHANNELS_TO_JOIN.chunks(CHANNELS_PER_LISTENER) {
                let twitch_listener = Arc::new(Listener::new(listener_transmitter.clone()));
                let listeners_lock = listeners.lock();
                listeners_lock.await.insert(
                    twitch_listener.username.to_string(),
                    twitch_listener.clone(),
                );
                match twitch_listener.clone().connect().await {
                    Ok(_) => println!("Successfully connected. - Not Actually - it is in process"),
                    Err(e) => {
                        eprintln!("Failed to connect: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
                tokio::spawn(async move {
                    for channel in chunk {
                        let twitch_listener_clone = Arc::clone(&twitch_listener); // Clone for each iteration
                        let username = twitch_listener_clone.username.to_string();
                        match twitch_listener_clone.join_channel(channel).await {
                            Ok(_) => println!("{} Joined channel {}", username, channel),
                            Err(e) => eprintln!("Failed to join channel {}: {}", channel, e),
                        }
                    }
                });
            }
            while loop_listener_clone.clone().get_state() != WebSocketState::Disconnected {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    let twitch_account = get_env_variable("TWITCHACCOUNT", "twitchusername");
    let twitch_oauth = get_env_variable("TWITCHOAUTH", "oauth:1234p1234p1234p1234p1234p1234p");
    let replier = Arc::new(Replier::new(&twitch_account, &twitch_oauth));

    // Spawn a task for replying to messages.
    tokio::spawn(async move {
        let replier_clone = Arc::clone(&replier);
        let loop_replier_clone = Arc::clone(&replier);
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
        while let Some(message) = rxreplier_clone.recv().await {
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
