use data::channel_data::{Channel, ChannelState};
use futures_util::pin_mut;
use futures_util::{stream::StreamExt, SinkExt};
use rand::Rng;
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub mod data;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    NotConnected,
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

impl From<usize> for WebSocketState {
    fn from(value: usize) -> Self {
        match value {
            x if x == WebSocketState::NotConnected as usize => WebSocketState::NotConnected,
            x if x == WebSocketState::Connecting as usize => WebSocketState::Connecting,
            x if x == WebSocketState::Connected as usize => WebSocketState::Connected,
            _ => WebSocketState::Disconnected,
        }
    }
}
pub struct WebSocketClient {
    pub tx: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    websocket_state: Arc<AtomicUsize>,
    channels: Mutex<VecDeque<Channel>>,
}

impl WebSocketClient {
    pub fn new() -> Self {
        WebSocketClient {
            tx: Mutex::new(None),
            websocket_state: Arc::new(AtomicUsize::new(WebSocketState::NotConnected as usize)),
            channels: Mutex::new(VecDeque::new()),
        }
    }

    pub fn get_state(&self) -> WebSocketState {
        match self.websocket_state.load(Ordering::SeqCst) {
            x if x == WebSocketState::NotConnected as usize => WebSocketState::NotConnected,
            x if x == WebSocketState::Connecting as usize => WebSocketState::Connecting,
            x if x == WebSocketState::Connected as usize => WebSocketState::Connected,
            x if x == WebSocketState::Failed as usize => WebSocketState::Failed,
            _ => WebSocketState::Disconnected,
        }
    }

    fn set_state(&self, state: WebSocketState) {
        self.websocket_state.store(state as usize, Ordering::SeqCst);
    }

    pub async fn connect_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.get_state() {
            WebSocketState::NotConnected => {
                self.set_state(WebSocketState::Connecting);
            }
            WebSocketState::Connecting => return Ok(()),
            WebSocketState::Connected => return Ok(()),
            WebSocketState::Disconnected => {
                println!("Disconnected, trying to re-connect");
                self.set_state(WebSocketState::Connecting)
            }
            WebSocketState::Failed => todo!(),
        }

        let url = "ws://irc-ws.chat.twitch.tv:80";
        println!("Attempting to connect to WebSocket at URL: {}", url);

        let ws_stream = match connect_async(url).await {
            Ok(stream) => stream.0,
            Err(e) => {
                println!("Failed to connect: {:?}", e);
                self.set_state(WebSocketState::Failed);
                return Err(Box::new(e));
            }
        };

        let (mut write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();

        *self.tx.lock().await = Some(tx.clone());

        let nick_message = format!(
            "NICK justinfan{}",
            rand::thread_rng().gen_range(10000..=99999)
        );

        self.send_message(&nick_message).await?;
        self.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await?;

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    println!("Error sending message: {:?}", e);
                }
            }
        });

        self.set_state(WebSocketState::Connected);
        println!("WebSocket Connected and ready.");

        self.join_channel("chapterverse").await;

        // Infinite loop to listen to incoming messages.
        self.listen_for_messages_while_loop(read, tx).await;

        Ok(())
    }

    pub async fn join_channel(&self, channel_name: &str) {
        {
            let mut channels = self.channels.lock().await;
            if !channels.iter().any(|c| c.name == channel_name) {
                let new_channel = Channel {
                    name: channel_name.to_string(),
                    state: ChannelState::NotConnected,
                };
                channels.push_back(new_channel);
                println!("Added channel: {}", channel_name);
            } else {
                println!("Channel already exists: {}", channel_name);
            }
        }
        self.join_pending_channels().await;
    }

    async fn join_pending_channels(&self) {
        if self.get_state() != WebSocketState::Connected {
            println!("WebSocket is not connected. Unable to join channels.");
            return;
        }
        let mut channels = self.channels.lock().await;

        for channel in channels.iter_mut() {
            if channel.state == ChannelState::NotConnected {
                // Attempt to send the JOIN message for the channel
                if let Err(e) = self.send_message(&format!("JOIN #{}", channel.name)).await {
                    println!("Error joining channel {}: {}", channel.name, e);
                } else {
                    println!("Joining channel: {}", channel.name);
                    channel.state = ChannelState::Connecting; // Update the state to reflect the action
                }
            } else {
                println!("Already joined channel: {}", channel.name);
            }
        }
    }

    async fn listen_for_messages_while_loop(
        &self,
        read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>,
        _tx: mpsc::UnboundedSender<Message>,
    ) {
        pin_mut!(read); // This pins the read stream to the stack
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    self.handle_message(text).await;
                }
                Err(e) => {
                    self.set_state(WebSocketState::Disconnected);
                    let mut channels = self.channels.lock().await;
                    for channel in channels.iter_mut() {
                        channel.state = ChannelState::NotConnected;
                    }
                    println!("Message Error: {:?}", e)
                }
                _ => {}
            }
        }
    }

    async fn handle_message(&self, message: String) {
        if message.starts_with("PING") {
            println!("Received PING, sending PONG.");
            let _ = self.send_message(&message.replace("PING", "PONG")).await;
        } else if message.contains("PRIVMSG") {
            println!("Received PRIVMSG: {}", message);
            if let Some(parsed_message) = crate::data::message_data::parse_message(&message) {
                println!("Message: {}", parsed_message.text);
            } else {
                println!("Failed to parse the message.");
            }
        } else if message.contains(":Welcome, GLHF!") {
            println!("Ready to listen to Twitch channels.");
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<(), &'static str> {
        let message = Message::Text(message.to_string());
        let tx_lock = self.tx.lock().await;
        if let Some(tx) = &*tx_lock {
            tx.send(message).map_err(|_| "Failed to send message")
        } else {
            Err("Connection not initialized")
        }
    }
}
