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
        loop {
            match self.websocket.get_state() {
                WebSocketState::NotConnected | WebSocketState::Disconnected => {
                    println!("Attempting to connect or reconnect...");
                    self.websocket.set_state(WebSocketState::Connecting);
                }
                WebSocketState::Connecting | WebSocketState::Connected => {
                    println!("Already connecting or connected, no action taken.");
                    return Ok(());
                }
                WebSocketState::Failed => {
                    println!("Previous attempt failed, trying again...");
                    // Optional: Implement logic to handle a permanent failure state if necessary
                }
            }

            let url = "ws://irc-ws.chat.twitch.tv:80";
            match connect_async(url).await {
                Ok((ws_stream, _)) => {
                    let self_clone = self.clone();
                    self_clone.handle_connection_success(ws_stream).await;
                    println!("WebSocket Connected and ready.");
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    self.websocket.set_state(WebSocketState::Disconnected);

                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    // Continues the loop to retry connection
                }
            }
        }

        Ok(())
    }
    async fn handle_connection_success(
        self: Arc<Self>,
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) {
        let (mut write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        *self.websocket.write.lock().await = Some(tx.clone());

        let justinfan = format!(
            "NICK justinfan{}",
            rand::thread_rng().gen_range(10000..=99999)
        );
        if let Err(e) = self.websocket.send_message(&justinfan).await {
            eprintln!("Error sending registration message: {}", e);
            return;
        }

        if let Err(e) = self.websocket.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await {
            eprintln!("Error sending CAP REQ message: {}", e);
            return;
        }

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    eprintln!("Error sending message: {:?}", e);
                }
            }
        });

        //self.websocket.join_pending_channels().await;

        let listener_clone = self.clone(); // Add this line before spawning async tasks
        tokio::spawn(async move {
            listener_clone.listen_for_messages(read).await; // Use `listener_clone` here
        });
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
