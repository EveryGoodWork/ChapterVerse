use crate::common::message_data::MessageData;
use futures_util::{stream::StreamExt, SinkExt};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use super::client::{WebSocket, WebSocketState};

pub struct Listener {
    pub message_tx: mpsc::UnboundedSender<MessageData>,
    websocket: WebSocket,
}

impl Listener {
    pub fn new(message_tx: mpsc::UnboundedSender<MessageData>) -> Self {
        Listener {
            message_tx: message_tx.clone(),
            websocket: WebSocket::new(message_tx),
        }
    }
    pub async fn connect(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send>> {
        match self.websocket.get_state() {
            WebSocketState::NotConnected => {
                println!("NotConnected, trying to connect");
                self.websocket.set_state(WebSocketState::Connecting);
            }
            WebSocketState::Connecting => return Ok(()),
            WebSocketState::Connected => return Ok(()),
            WebSocketState::Disconnected => {
                println!("Disconnected, trying to re-connect");
                self.websocket.set_state(WebSocketState::Connecting);
            }
            WebSocketState::Failed => todo!(),
        }
        let url = "ws://irc-ws.chat.twitch.tv:80";
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

        let (mut write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel(); // Use the receiver `rx` that is created here
        *self.websocket.write.lock().await = Some(tx.clone());

        let justinfan = format!(
            "NICK justinfan{}",
            rand::thread_rng().gen_range(10000..=99999)
        );
        self.websocket.send_message(&justinfan).await.map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
                as Box<dyn std::error::Error + Send>
        })?;
        //self.websocket.send_message(&justinfan).await?;
        self.websocket.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await
    .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error + Send>)?;

        //self.websocket.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await?;

        // Correctly use the receiver in a separate task for message sending
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    println!("Error sending message: {:?}", e);
                }
            }
        });

        self.websocket.set_state(WebSocketState::Connected);
        println!("WebSocket Connected and ready.");
        self.websocket.join_pending_channels().await;

        // Spawn listen_for_messages as a separate task
        // You should ensure `self` can be safely shared or cloned across tasks. If not, consider using Arc<Mutex<Self>>.
        let listener = self.clone(); // Adjust according to your struct's capabilities
        tokio::spawn(async move {
            listener.listen_for_messages(read).await;
        });

        Ok(())
    }

    pub async fn join_channel(&self, channel_name: &str) -> Result<(), &'static str> {
        println!("join_channel {}", channel_name);
        self.websocket.join_channel(channel_name).await;
        Ok(())
    }
    pub fn get_state(&self) -> WebSocketState {
        self.websocket.get_state()
    }
    async fn listen_for_messages(
        &self,
        read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Send
            + 'static,
    ) {
        self.websocket.listen_for_messages(read).await;
    }
}
