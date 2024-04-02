use super::client::{WebSocket, WebSocketState};
use crate::common::message_data::MessageData;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Listener {
    pub message_tx: mpsc::UnboundedSender<MessageData>,
    websocket: Arc<WebSocket>,
}
impl Listener {
    pub fn new(message_tx: mpsc::UnboundedSender<MessageData>) -> Self {
        let username = format!("justinfan{}", rand::thread_rng().gen_range(10000..=99999));
        Listener {
            message_tx: message_tx.clone(),
            websocket: WebSocket::new(message_tx, username, None),
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
        self.websocket.clone().join_channel(channel_name).await;
        Ok(())
    }
    pub fn get_state(&self) -> WebSocketState {
        self.websocket.get_state()
    }
}
