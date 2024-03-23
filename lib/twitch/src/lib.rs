use futures_util::pin_mut;
use futures_util::{stream::StreamExt, SinkExt};
use rand::Rng;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub struct WebSocketClient {
    pub tx: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    websocket_state: Arc<Mutex<WebSocketState>>,
}

impl WebSocketClient {
    pub fn new(websocket_state: Arc<Mutex<WebSocketState>>) -> Self {
        WebSocketClient {
            tx: Mutex::new(None),
            websocket_state,
        }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = "ws://irc-ws.chat.twitch.tv:80";
        println!("Attempting to connect to WebSocket at URL: {}", url);
        {
            let ws_state = self.websocket_state.lock().await;
            if ws_state.is_connecting.load(Ordering::SeqCst) {
                println!("Already attempting to connect. Exiting...");
                return Ok(());
            } else {
                ws_state.is_connecting.store(true, Ordering::SeqCst);
                println!("Connection attempt marked as in progress.");
            }
        }

        let ws_stream = connect_async(url)
            .await
            .map_err(|e| {
                println!("Failed to connect: {:?}", e);
                e
            })?
            .0;

        let (mut write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();

        *self.tx.lock().await = Some(tx.clone());

        let nick_message = format!(
            "NICK justinfan{}",
            rand::thread_rng().gen_range(10000..=99999)
        );

        self.send_message(&nick_message).await?;
        self.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await?;
        self.send_message("JOIN #chapterverse").await?;

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    println!("Error sending message: {:?}", e);
                }
            }
        });

        self.listen_to_incoming_messages(read, tx).await;
        Ok(())
    }

    async fn listen_to_incoming_messages(
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
                Err(e) => println!("Error reading message: {:?}", e),
                _ => {} // Ignore other cases
            }
        }
    }

    async fn handle_message(&self, message: String) {
        if message.starts_with("PING") {
            println!("Handling PING message.");
            if let Err(e) = self.send_message("PONG").await {
                println!("Failed to respond to PING with PONG: {:?}", e);
            }
        } else if message.contains("PRIVMSG") {
            println!("Received PRIVMSG: {}", message);
        } else if message.contains(":Welcome, GLHF!") {
            println!("Connected to Twitch");
        }
    }

    pub async fn send_message(&self, message_str: &str) -> Result<(), &'static str> {
        let message = Message::Text(message_str.to_string());
        let tx_lock = self.tx.lock().await;
        if let Some(tx) = &*tx_lock {
            tx.send(message).map_err(|_| "Failed to send message")
        } else {
            Err("Connection not initialized")
        }
    }
}

pub struct WebSocketState {
    is_connecting: AtomicBool,
}

impl WebSocketState {
    pub fn new() -> Self {
        WebSocketState {
            is_connecting: AtomicBool::new(false),
        }
    }
}
