use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{pin_mut, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

use crate::common;
use crate::common::channel_data::{Channel, ChannelState};
use crate::common::message_data::MessageData;

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
            x if x == WebSocketState::Failed as usize => WebSocketState::Failed,
            _ => WebSocketState::Disconnected,
        }
    }
}

pub struct WebSocket {
    pub websocket_state: Arc<AtomicUsize>,
    pub write: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    channels: Arc<Mutex<VecDeque<Channel>>>,
    message_tx: Option<mpsc::UnboundedSender<MessageData>>,
}
impl Clone for WebSocket {
    fn clone(&self) -> Self {
        WebSocket {
            websocket_state: Arc::clone(&self.websocket_state),
            write: Arc::clone(&self.write),
            channels: Arc::clone(&self.channels),
            message_tx: self.message_tx.clone(),
        }
    }
}
impl WebSocket {
    pub fn new(message_tx: mpsc::UnboundedSender<MessageData>) -> Self {
        Self {
            message_tx: Some(message_tx),
            websocket_state: Arc::new(AtomicUsize::new(WebSocketState::NotConnected as usize)),
            write: Arc::new(Mutex::new(None)),
            channels: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn get_state(&self) -> WebSocketState {
        WebSocketState::from(self.websocket_state.load(Ordering::SeqCst))
    }
    pub async fn set_state(&self, state: WebSocketState) {
        self.websocket_state.store(state as usize, Ordering::SeqCst);
        if state == WebSocketState::Disconnected {
            let mut channels = self.channels.lock().await;
            for channel in channels.iter_mut() {
                channel.state = ChannelState::NotConnected;
            }
        }
    }
    pub async fn send_message(&self, message: &str) -> Result<(), &'static str> {
        let msg = Message::Text(message.to_string());
        let write_guard = self.write.lock().await;
        if let Some(tx) = &*write_guard {
            tx.send(msg).map_err(|_| "Failed to send message")
        } else {
            Err("Connection not initialized")
        }
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
            } else {
                println!("Channel already exists: {}", channel_name);
            }
            println!("join_channel channels queue size: {}", channels.len()); // Print the size of the channels queue
        }
        self.join_pending_channels().await;
    }

    pub async fn join_pending_channels(&self) {
        println!("WebSocketState: {:?}", self.get_state());

        if self.get_state() == WebSocketState::Connected {
            // Proceed with joining channels if already connected
            self.process_channel_joining().await;
        } else {
            println!("WebSocket is not connected. Waiting to join channels.");
            // Spawn a new asynchronous task to wait and retry
            tokio::spawn({
                let ws_clone = self.clone();
                async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        if ws_clone.get_state() == WebSocketState::Connected {
                            println!("WebSocket is now connected. Attempting to join channels.");
                            ws_clone.process_channel_joining().await;
                            break;
                        }
                    }
                }
            });
        }
    }

    async fn process_channel_joining(&self) {
        let mut channels = self.channels.lock().await;
        println!("Current channels queue size: {}", channels.len()); // Print the size of the channels queue

        for channel in channels.iter_mut() {
            if channel.state == ChannelState::NotConnected {
                if let Err(e) = self.send_message(&format!("JOIN #{}", channel.name)).await {
                    println!("Error joining channel {}: {}", channel.name, e);
                } else {
                    println!("Joining channel: {}", channel.name);
                    channel.state = ChannelState::Connecting; // Update the state
                }
            } else {
                println!("Already joined channel: {}", channel.name);
            }
        }
    }

    pub async fn listen_for_messages(
        &self,
        read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>,
    ) {
        pin_mut!(read); // Pin the stream to the stack
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    println!("Received message: {}", text);
                    if text.starts_with("PING") {
                        println!("Received PING, sending: PONG");
                        self.send_message(&text.replace("PING", "PONG")).await.ok();
                    } else if text.contains("PRIVMSG") {
                        if let Some(parsed_message) = common::message_data::parse_message(&text) {
                            if let Some(sender) = &self.message_tx {
                                if let Err(e) = sender.send(parsed_message) {
                                    eprintln!("Failed to send message to main.rs: {}", e);
                                }
                            }
                        }
                    } else if !text.contains("PRIVMSG") & text.contains(":Welcome, GLHF!") {
                        println!(":Welcome, GLHF!");
                        self.set_state(WebSocketState::Connected).await;
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {:?}", e);
                    if let tokio_tungstenite::tungstenite::Error::Io(io_error) = &e {
                        if io_error.kind() == std::io::ErrorKind::ConnectionReset {
                            eprintln!("Connection was reset by the remote host.");
                            self.set_state(WebSocketState::Disconnected).await;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
#[async_trait]
pub trait Client {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error>>;
}
