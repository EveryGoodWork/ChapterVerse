use crate::common;
use crate::common::channel_data::{Channel, ChannelState};
use crate::common::message_data::MessageData;
use async_trait::async_trait;
use futures_util::{pin_mut, SinkExt, StreamExt};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

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
    pub username: String,
    pub oauth_token: Option<String>,
    pub websocket_state: Arc<AtomicUsize>,
    pub write: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    channels: Arc<Mutex<VecDeque<Channel>>>,
    message_tx: Option<mpsc::UnboundedSender<MessageData>>,
    message_queue: Arc<Mutex<VecDeque<String>>>,
    message_notifier: Arc<Notify>,
    last_message_sent_at: Arc<Mutex<Instant>>,
}

impl WebSocket {
    pub fn new(
        message_tx: mpsc::UnboundedSender<MessageData>,
        username: String,
        oauth_token: impl Into<Option<String>>,
    ) -> Arc<Self> {
        //pub fn new(message_tx: mpsc::UnboundedSender<MessageData>) -> Arc<Self> {
        let instance = Arc::new(Self {
            username,
            oauth_token: oauth_token.into(),
            message_tx: Some(message_tx),
            websocket_state: Arc::new(AtomicUsize::new(WebSocketState::NotConnected as usize)),
            write: Arc::new(Mutex::new(None)),
            channels: Arc::new(Mutex::new(VecDeque::new())),
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            message_notifier: Arc::new(Notify::new()),
            last_message_sent_at: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(30))),
        });

        // Start processing the message queue in a background task
        let instance_clone = Arc::clone(&instance);
        spawn(async move {
            instance_clone.process_message_queue().await;
        });

        instance
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

    pub async fn connect(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match self.get_state() {
                WebSocketState::NotConnected | WebSocketState::Disconnected => {
                    println!("Attempting to connect or reconnect...");
                    self.set_state(WebSocketState::Connecting).await;
                }
                WebSocketState::Connecting | WebSocketState::Connected => {
                    println!("Already connecting or connected, no action taken.");
                    return Ok(());
                }
                WebSocketState::Failed => {
                    println!("Previous attempt failed, trying again...");
                }
            }
            let url = "ws://irc-ws.chat.twitch.tv:80";
            match connect_async(url).await {
                Ok((ws_stream, _)) => {
                    self.handle_connection_success(ws_stream).await;
                    println!("WebSocket Connected and ready.");
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to connect: {:?}", e);
                    self.set_state(WebSocketState::Disconnected).await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
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
        *self.write.lock().await = Some(tx.clone());

        if let Some(oauth) = &self.oauth_token {
            let auth_message = format!("PASS {}", oauth);
            self.send_message(&auth_message).await;
        }

        let nick_message = format!("NICK {}", self.username);
        self.send_message(&nick_message).await;

        //self.websocket.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership twitch.tv/subscriptions twitch.tv/bits twitch.tv/badges").await;
        //This is the minimal meta necessary to process messages for ChapterVerse.
        self.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands")
            .await;

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    eprintln!("Error sending message: {:?}", e);
                }
            }
        });
        let listener_clone = self.clone();
        tokio::spawn(async move {
            listener_clone.listen_for_messages(read).await;
        });
    }
    pub async fn send_message(&self, message: &str) {
        let mut queue = self.message_queue.lock().await;
        queue.push_back(message.to_string());
        self.message_notifier.notify_one();
    }

    pub async fn process_message_queue(&self) {
        let rate_limit_interval = Duration::from_secs(30) / 20;
        loop {
            let now = Instant::now();
            let sleep_duration = {
                let mut last_sent = self.last_message_sent_at.lock().await;
                if now - *last_sent >= rate_limit_interval {
                    let mut queue = self.message_queue.lock().await;
                    if let Some(message) = queue.pop_front() {
                        let msg = Message::Text(message.to_string());
                        let write = Arc::clone(&self.write);
                        tokio::spawn(async move {
                            let write_guard = write.lock().await;
                            if let Some(tx) = &*write_guard {
                                if let Err(e) = tx.send(msg) {
                                    eprintln!("Failed to send message: {}", e);
                                }
                            } else {
                                eprintln!("Connection not initialized");
                            }
                        });
                        *last_sent = now;
                        None
                    } else {
                        drop(last_sent);
                        drop(queue);
                        self.message_notifier.notified().await;
                        None
                    }
                } else {
                    Some(rate_limit_interval - (now - *last_sent))
                }
            };
            if let Some(duration) = sleep_duration {
                tokio::time::sleep(duration).await;
                //println!("---Slept for rate limit, rechecking message queue");
            }
        }
    }

    pub async fn join_channel(self: Arc<Self>, channel_name: &str) {
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
            println!("join_channel channels queue size: {}", channels.len());
        }
        self.join_pending_channels().await;
    }

    pub async fn join_pending_channels(self: Arc<Self>) {
        println!("WebSocketState: {:?}", self.get_state());

        if self.get_state() == WebSocketState::Connected {
            self.process_channel_joining().await;
        } else {
            println!("WebSocket is not connected. Waiting to join channels.");
            tokio::spawn({
                let ws_clone = Arc::clone(&self);
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
                self.send_message(&format!("JOIN #{}", channel.name)).await;
                channel.state = ChannelState::Connected;
                println!("Joining channel: {}", channel.name);
                // TODO!  This should be a Connecting state that is then updated when a message is recieved that it's connected to the channel.
                //channel.state = ChannelState::Connecting;
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
                    println!("Message RAW: {}", text);
                    if text.starts_with("PING") {
                        println!("Received PING, sending: PONG");
                        self.send_message(&text.replace("PING", "PONG")).await;
                    } else if text.contains(" PRIVMSG #") {
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
