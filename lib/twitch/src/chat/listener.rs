use super::client::{WebSocket, WebSocketState};
use crate::common::message_data::MessageData;
use rand::Rng;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc;

pub struct Listener {
    pub message_tx: mpsc::UnboundedSender<MessageData>,
    pub websocket: Arc<WebSocket>,
    pub username: Arc<String>,
    pub channels_count: Arc<AtomicUsize>,
}
impl Listener {
    pub fn new(
        message_tx: mpsc::UnboundedSender<MessageData>,
        username: Option<String>,
        oauth_token: Option<String>,
    ) -> Self {
        let username = username
            .unwrap_or_else(|| format!("justinfan{}", rand::thread_rng().gen_range(10000..=99999)));
        Listener {
            message_tx: message_tx.clone(),
            websocket: WebSocket::new(message_tx, username.clone(), oauth_token),
            username: username.into(),
            channels_count: Arc::new(AtomicUsize::new(0)),
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
        let websocket_clone = Arc::clone(&self.websocket);
        websocket_clone.join_channel(channel_name).await;
        self.channels_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    pub async fn leave_channel(&self, channel_name: &str) -> Result<(), &'static str> {
        let websocket_clone = Arc::clone(&self.websocket);
        websocket_clone.leave_channel(channel_name).await;
        self.channels_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
    pub fn get_state(&self) -> WebSocketState {
        self.websocket.get_state()
    }
}
