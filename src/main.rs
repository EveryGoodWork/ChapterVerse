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

    // TODO:  Create a config files to pull these from, each channel gets it's own file.
    let channels_to_join = vec!["chapterverse".to_string(), "missionarygamer".to_string()];

    let (txl, rxl) = mpsc::unbounded_channel::<MessageData>();
    // TODO! This needs to be addressed as the replier doesn't really use this channel, but will for account messages.
    let (_txr, _rxr) = mpsc::unbounded_channel::<MessageData>();

    //This channel is for sending replies to the Twitch reply, not to be confused with the messages coming into the reply on their own.
    let (txreplier, rxreplier) = mpsc::unbounded_channel::<MessageData>();

    println!("Trying to connect");

    let listener = Arc::new(Listener::new(txl));

    let username = get_env_variable("USERNAME", "twitchusername");
    let oauth = get_env_variable("OAUTH", "oauth:1234p1234p1234p1234p1234p1234p");

    //This channel is for sending replies to the Twitch reply, not to be confused with the messages coming into the reply on their own.
    let txreplier_clone = txreplier.clone();
    let mut rxreplier_clone = rxreplier;

    let replier = Arc::new(Replier::new(_txr, &username, &oauth));
    // Assuming `channels_to_join` is cloned or moved into the async block appropriately
    let channels_clone = channels_to_join.clone();

    //THIS IS THE CODE TO REPLY
    tokio::spawn(async move {
        let replier_clone = Arc::clone(&replier);
        let loop_replier_clone = Arc::clone(&replier);
        match replier_clone.clone().connect().await {
            Ok(_) => {
                println!("Successfully connected.");
                // TODO! This is an initial message to show it's connected to the channel.
                let _ = replier_clone
                    .send_message("missionarygamer", "Jesus is Lord!")
                    .await;
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

    // Spawn a task to manage connection, listening, and reconnection
    tokio::spawn(async move {
        let listener_clone = Arc::clone(&listener);
        loop {
            let loop_listener_clone = Arc::clone(&listener_clone);
            match loop_listener_clone.connect().await {
                Ok(_) => println!("Successfully connected."),
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            // let loop_replier_clone = Arc::clone(&replier_clone);

            // Join channels
            // TODO! This will be pulled from config files.
            for channel in &channels_clone {
                match listener_clone.clone().join_channel(channel).await {
                    Ok(_) => println!("Successfully joined channel: {}", channel),
                    Err(e) => eprintln!("Failed to join channel {}: {}", channel, e),
                }
            }

            while listener_clone.get_state() != WebSocketState::Disconnected {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    let mut rx_clone = rxl;

    //This Listens for incoming messages.
    tokio::spawn(async move {
        while let Some(message) = rx_clone.recv().await {
            // TODO! Add a preliminary scan to determine if there is potential scripture(s) in this message.
            // TODO! this will pull from a user preference variable
            // TODO! Pull list of names to ignore from a configuraiton file
            if message.display_name != Some("ChapterVerse")
                && message.display_name != Some("EveryGoodWork")
            {
                println!("FUllMessage: {:?}", message);
                let bible_name_to_use = "KJV";
                if let Some(bible_arc) = BIBLES.get(bible_name_to_use) {
                    let bible: &Bible = &*bible_arc;
                    let scripture_message = match bible.get_scripture(&message.text) {
                        Some(verse) => format!("{}", verse.scripture),
                        None => "Verse not found".to_string(),
                    };
                    PrintCommand::Info.print_message(
                        &format!(
                            "Bible {}, {}",
                            bible_name_to_use,
                            message.display_name.unwrap_or_default()
                        ),
                        &scripture_message,
                    );
                    println!("Send Message Here");
                    // Assuming `message` is an instance of `MessageData`
                    let mut reply_message_clone = message.clone();
                    reply_message_clone.text = scripture_message;
                    if let Err(e) = txreplier_clone.send(reply_message_clone) {
                        eprintln!("Failed to send cloned message: {}", e);
                    }
                } else {
                    eprintln!("Bible named '{}' not found.", bible_name_to_use);
                }
            }
            match message.complete() {
                Ok(duration) => println!("Message processing duration: {:?}", duration),
                Err(e) => eprintln!("Error calculating duration: {}", e),
            }
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
