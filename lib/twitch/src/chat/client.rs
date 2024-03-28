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
    channels: Mutex<VecDeque<Channel>>,
    message_tx: Option<mpsc::UnboundedSender<MessageData>>,
}
impl Clone for WebSocket {
    fn clone(&self) -> Self {
        WebSocket {
            websocket_state: Arc::clone(&self.websocket_state),
            write: Arc::clone(&self.write),
            channels: Mutex::new(VecDeque::new()),
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
            channels: Mutex::new(VecDeque::new()),
        }
    }
    pub fn get_state(&self) -> WebSocketState {
        WebSocketState::from(self.websocket_state.load(Ordering::SeqCst))
    }

    pub fn set_state(&self, state: WebSocketState) {
        self.websocket_state.store(state as usize, Ordering::SeqCst);
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
                println!("Added channel: {}", channel_name);
            } else {
                println!("Channel already exists: {}", channel_name);
            }
        }
        self.join_pending_channels().await;
    }

    pub async fn join_pending_channels(&self) {
        println!("WebSocketState: {:?}", self.get_state());

        if self.get_state() != WebSocketState::Connected {
            println!("WebSocket is not connected. Unable to join channels.");
            return;
        }

        let mut channels = self.channels.lock().await;

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
                    // Handle the message, e.g., parse it and act accordingly
                    if text.starts_with("PING") {
                        self.send_message(&text.replace("PING", "PONG")).await.ok();
                    } else if text.contains("PRIVMSG") {
                        if let Some(parsed_message) = common::message_data::parse_message(&text) {
                            if let Some(sender) = &self.message_tx {
                                if let Err(e) = sender.send(parsed_message) {
                                    eprintln!("Failed to send message to main.rs: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {:?}", e);
                    if let tokio_tungstenite::tungstenite::Error::Io(io_error) = &e {
                        if io_error.kind() == std::io::ErrorKind::ConnectionReset {
                            eprintln!("Connection was reset by the remote host.");
                            // Update the connection state to Disconnected
                            self.set_state(WebSocketState::Disconnected);
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
