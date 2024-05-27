use super::client::{WebSocket, WebSocketState};
use crate::common::message_data::MessageData;
use std::sync::Arc;
use tokio::sync::mpsc;

/// TODO!  TRAP FOR THIS ERROR: @msg-id=msg_requires_verified_phone_number :tmi.twitch.tv NOTICE #missionarygamer :A verified phone number is required to chat in this channel. Please visit https://www.twitch.tv/settings/security to verify your phone number.
pub struct Replier {
    websocket: Arc<WebSocket>,
}
impl Replier {
    pub fn new(
        message_tx: mpsc::UnboundedSender<MessageData>,
        username: &str,
        oauth: &str,
    ) -> Self {
        Replier {
            websocket: WebSocket::new(message_tx, username.to_string(), oauth.to_string()),
        }
    }

    pub async fn connect(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send>> {
        self.websocket
            .clone()
            .connect()
            .await
            .expect("Failed to connect");
        Ok(())
    }

    pub async fn join_channel(self: Arc<Self>, channel_name: &str) -> Result<(), &'static str> {
        println!("Replier join_channel: {}", channel_name);
        self.websocket.clone().join_channel(channel_name).await;
        Ok(())
    }

    pub fn get_state(&self) -> WebSocketState {
        self.websocket.get_state()
    }

    pub async fn send_message(
        self: Arc<Self>,
        channel_name: &str,
        message_text: &str,
    ) -> Result<(), &'static str> {
        let message_data = MessageData {
            channel: channel_name.to_string(),
            text: message_text.chars().take(500).collect(),
            ..MessageData::default()
        };
        self.websocket.send_message(message_data).await;
        Ok(())
    }

    pub async fn reply_message(self: Arc<Self>, message: MessageData) {
        self.websocket.send_message(message).await;
    }
}
