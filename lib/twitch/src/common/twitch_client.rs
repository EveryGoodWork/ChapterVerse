use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

use super::channel_data::Channel;
use crate::common::channel_data::ChannelState;

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
}
impl Clone for WebSocket {
    fn clone(&self) -> Self {
        WebSocket {
            websocket_state: Arc::clone(&self.websocket_state),
            write: Arc::clone(&self.write),
            // For the channels, since we cannot directly clone a Mutex,
            // you might consider initializing a new Mutex with a new VecDeque,
            // or appropriately managing the state according to your app's logic.
            channels: Mutex::new(VecDeque::new()),
        }
    }
}
impl WebSocket {
    pub fn new() -> Self {
        Self {
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
}

#[async_trait]
pub trait TwitchClient {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error>>;
    //async fn send_message(&self, message: &str) -> Result<(), &'static str>;
    //async fn join_channel(&self, channel_name: &str) -> Result<(), &'static str>;

    // fn get_atomic_state(&self) -> Arc<AtomicUsize>;
    // fn get_state(&self) -> WebSocketState {
    //     match self.get_atomic_state().load(Ordering::SeqCst) {
    //         x if x == WebSocketState::NotConnected as usize => WebSocketState::NotConnected,
    //         x if x == WebSocketState::Connecting as usize => WebSocketState::Connecting,
    //         x if x == WebSocketState::Connected as usize => WebSocketState::Connected,
    //         x if x == WebSocketState::Failed as usize => WebSocketState::Failed,
    //         _ => WebSocketState::Disconnected,
    //     }
    // }
}
