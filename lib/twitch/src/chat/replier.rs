use super::client::{WebSocket, WebSocketState};
use crate::common::message_data::MessageData;
use std::sync::Arc;
use tokio::sync::mpsc;

/// TODO!  TRAP FOR THIS ERROR: @msg-id=msg_requires_verified_phone_number :tmi.twitch.tv NOTICE #missionarygamer :A verified phone number is required to chat in this channel. Please visit https://www.twitch.tv/settings/security to verify your phone number.
pub struct Replier {
    websocket: Arc<WebSocket>,
}
impl Replier {
    pub fn new(username: &str, oauth: &str) -> Self {
        let (message_tx, _message_rx) = mpsc::unbounded_channel::<MessageData>();
        Replier {
            //message_tx: message_tx.clone(),
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
        println!("join_channel {}", channel_name);
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
        // Directly creating the MessageData object with provided values
        let message_data = MessageData {
            channel: channel_name.to_string(),
            text: message_text.to_string(),
            raw_message: format!("PRIVMSG #{} :{}\r\n", channel_name, message_text),
            ..MessageData::default()
        };

        println!("---DEBUG SendMessage: {:?}", message_data);
        self.websocket.clone().send_message(message_data).await;

        Ok(())
    }
}
