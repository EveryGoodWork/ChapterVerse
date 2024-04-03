use bible::scripture::bible::Bible;
use helpers::print_color::PrintCommand;
use tokio::sync::mpsc;

use futures::future::pending;
use std::env;
use std::sync::Arc;
use twitch::chat::client::WebSocketState;
use twitch::chat::Listener;
use twitch::chat::Replier;
use twitch::common::message_data::MessageData;
use twitch::common::message_data::Type;

use crate::helpers::env_variables::get_env_variable;
use crate::helpers::statics::BIBLES;

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
    let twitch_listener = Arc::new(Listener::new(listener_transmitter));
    let (txreplier, rxreplier) = mpsc::unbounded_channel::<MessageData>();
    let txreplier_clone = txreplier.clone();
    let mut rxreplier_clone = rxreplier;

    // TODO!  Create a config files to pull these from, each channel gets it's own file.
    let channels_to_join = vec!["chapterverse".to_string(), "missionarygamer".to_string()];

    tokio::spawn(async move {
        //This Listens for incoming Twitch messages.
        while let Some(message) = listener_reciever.recv().await {
            // PrintCommand::Info.print_message(
            //     "*listener_reciever_channel.recv().await",
            //     &message.raw_message,
            // );

            println!("Message Type:  {:?}", message.tags);
            if message.tags.contains(&Type::Ignore) {
                // println!("*IGNORE: {:?}", message.display_name);
            } else {
                for tag in &message.tags {
                    match tag {
                        Type::None => (),
                        Type::Command => {
                            println!("COMMAND!");
                            ()
                        }
                        Type::PossibleScripture => {
                            let bible_name_to_use = "KJV";
                            if let Some(bible_arc) = BIBLES.get(bible_name_to_use) {
                                let bible: &Bible = &*bible_arc;
                                println!("message.text: {}", message.text);
                                let scripture_message = match bible.get_scripture(&message.text) {
                                    Some(verse) => {
                                        message.clone().tags.push(Type::Scripture);
                                        let mut reply_message_clone = message.clone();
                                        reply_message_clone.text = format!(
                                            "{} - {} - {:?}",
                                            verse.scripture,
                                            verse.abbreviation,
                                            message.complete()
                                        );
                                        // TODO! Update this to use the reply field.
                                        //reply_message_clone.reply = Some(verse.scripture);
                                        if let Err(e) =
                                            txreplier_clone.send(reply_message_clone.clone())
                                        {
                                            eprintln!("Failed to send cloned message: {}", e);
                                        }
                                        reply_message_clone.text
                                    }
                                    None => {
                                        message.clone().tags.push(Type::NotScripture);
                                        "Verse not found".to_string()
                                    }
                                };
                                PrintCommand::Info.print_message(
                                    &format!(
                                        "Bible {}, {}",
                                        bible_name_to_use,
                                        message.display_name.unwrap_or_default()
                                    ),
                                    &scripture_message,
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
                }
            }
            match message.complete() {
                Ok(duration) => println!("Message processing duration: {:?}", duration),
                Err(e) => eprintln!("Error calculating duration: {}", e),
            }
        }
    });

    // Spawn a task to manage connection, listening, and reconnection
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
            for channel in &channels_to_join.clone() {
                match loop_listener_clone.clone().join_channel(channel).await {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to join channel {}: {}", channel, e),
                }
            }
            while loop_listener_clone.clone().get_state() != WebSocketState::Disconnected {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    let twitch_account = get_env_variable("TWITCHACCOUNT", "twitchusername");
    let twitch_oauth = get_env_variable("TWITCHOAUTH", "oauth:1234p1234p1234p1234p1234p1234p");
    let replier = Arc::new(Replier::new(&twitch_account, &twitch_oauth));

    // THIS IS THE CODE TO REPLY
    tokio::spawn(async move {
        let replier_clone = Arc::clone(&replier);
        let loop_replier_clone = Arc::clone(&replier);
        match replier_clone.clone().connect().await {
            Ok(_) => {
                println!("Successfully connected for Replying.");
                // TODO! This is an initial message to show it's connected to the channel.
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
            let _ = loop_replier_clone
                .clone()
                // TODO! Update MessageData with a reply_text field
                .send_message(&message.channel, &message.text)
                .await;
        }
    });

    // let mut rx_clone = listener_reciever_channel;

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
