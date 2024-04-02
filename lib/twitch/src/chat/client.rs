use crate::common;
use crate::common::channel_data::{Channel, ChannelState};
use crate::common::message_data::MessageData;
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::{pin_mut, SinkExt, StreamExt};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use futures_util::stream::SplitStream;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

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
    //New fields
    twitch_sink:
        Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WebSocketMessage>>>>,
    twitch_stream: Arc<Mutex<Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>>,
    twitch_channel_transmitter: Arc<Mutex<Option<UnboundedSender<WebSocketMessage>>>>,
    twitch_channel_reciever: Arc<Mutex<Option<UnboundedReceiver<WebSocketMessage>>>>,
}

impl WebSocket {
    pub fn new(
        message_tx: mpsc::UnboundedSender<MessageData>,
        username: String,
        oauth_token: impl Into<Option<String>>,
    ) -> Arc<Self> {
        let instance = Arc::new(Self {
            username,
            oauth_token: oauth_token.into(),
            message_tx: Some(message_tx),
            websocket_state: Arc::new(AtomicUsize::new(WebSocketState::NotConnected as usize)),
            write: Arc::new(Mutex::new(None)),
            twitch_sink: Arc::new(Mutex::new(None)),
            twitch_stream: Arc::new(Mutex::new(None)),
            twitch_channel_transmitter: Arc::new(Mutex::new(None)),
            twitch_channel_reciever: Arc::new(Mutex::new(None)),

            channels: Arc::new(Mutex::new(VecDeque::new())),
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            message_notifier: Arc::new(Notify::new()),
            last_message_sent_at: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(30))),
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

    fn extract_channel(message: &Message) -> Option<&str> {
        if let Message::Text(ref text) = message {
            let (_before, after) = text.split_once("PRIVMSG #")?;
            after.split_once(' ').map(|(channel, _)| channel)
        } else {
            None
        }
    }

    pub async fn connect(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match self.get_state() {
                WebSocketState::NotConnected | WebSocketState::Disconnected => {
                    println!("Attempting to connect...");
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
                    println!("---WebSocket Connected and ready.");
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
        let (split_sink, split_stream) = ws_stream.split();
        //TODO! Why are these created here.
        let (twitch_transmitter, twitch_reciever) = mpsc::unbounded_channel();

        *self.twitch_sink.lock().await = Some(split_sink);
        *self.twitch_stream.lock().await = Some(split_stream);
        *self.twitch_channel_transmitter.lock().await = Some(twitch_transmitter.clone());
        *self.twitch_channel_reciever.lock().await = Some(twitch_reciever);

        //*self.write.lock().await = Some(twitch_channel_write.clone());

        //Start the Channel for sending messages to twitch.
        tokio::spawn({
            let self_clone = self.clone();
            async move {
                while let Some(twitch_channel_read) =
                    self_clone.twitch_channel_reciever.lock().await.as_mut()
                {
                    while let Some(message) = twitch_channel_read.recv().await {
                        let mut sink = self_clone.twitch_sink.lock().await;
                        if let Some(sink) = sink.as_mut() {
                            if let Err(e) = sink.send(message).await {
                                eprintln!("Error sending message: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        //Start the Channel for recieving messages from twitch.
        // Corrected approach: Process the stream where it is accessible.
        let listener_clone = self.clone();
        tokio::spawn(async move {
            let mut twitch_stream_option = listener_clone.twitch_stream.lock().await;
            if let Some(twitch_stream) = twitch_stream_option.as_mut() {
                listener_clone.listen_for_messages(twitch_stream).await;
            }
        });

        if let Some(oauth) = &self.oauth_token {
            if let Some(twitch_channel_write) = &*self.twitch_channel_transmitter.lock().await {
                twitch_channel_write
                    .send(format!("PASS {}", oauth).into())
                    .unwrap_or_else(|e| eprintln!("Error sending message: {:?}", e));
            }
        }

        if let Some(twitch_channel_write) = &*self.twitch_channel_transmitter.lock().await {
            twitch_channel_write
                .send(format!("NICK {}", self.username).into())
                .unwrap_or_else(|e| eprintln!("Error sending message: {:?}", e));
            twitch_channel_write
                .send("CAP REQ :twitch.tv/tags twitch.tv/commands".into())
                .unwrap_or_else(|e| eprintln!("Error sending message: {:?}", e));
        }

        // twitch_channel_write
        //     .send(
        //         "CAP REQ :twitch.tv/tags twitch.tv/commands"
        //             .to_string()
        //             .into(),
        //     )
        //     .map_err(|e| eprintln!("Error sending message: {:?}", e))
        //     .ok();

        // // TESTING JOINING - THIS WORKS
        // twitch_channel_write
        //     .send("JOIN #chapterverse".to_string().into())
        //     .map_err(|e| eprintln!("Error sending message: {:?}", e))
        //     .ok();
    }

    pub async fn send_message(&self, message: MessageData) {
        println!("--send_message: {}", message.text);
        // if let Some(sender) = &self.message_tx {
        //     if let Err(e) = sender.send(message) {
        //         eprintln!("Failed to send message to main.rs: {}", e);
        //     }
        // }

        //TODO ! Determine how to send a full response message/ reply message etc, also implement the throttling hear
        //TODO ! Add a reply_text option to the MessageData

        let twitch_message = format!("PRIVMSG #{} :{}\r\n", message.channel, message.text);

        if let Some(transmitter_locked) = &*self.twitch_channel_transmitter.lock().await {
            transmitter_locked
                .send(twitch_message.into())
                .unwrap_or_else(|e| eprintln!("Error sending command message: {:?}", e));
        }
    }

    pub async fn send_command(&self, command_text: &str) {
        println!("send_command {}", command_text);

        // Accessing the `twitch_channel_write` field properly with lock.
        if let Some(transmitter_locked) = &*self.twitch_channel_transmitter.lock().await {
            transmitter_locked
                .send(command_text.into())
                .unwrap_or_else(|e| eprintln!("Error sending command message: {:?}", e));
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
            println!("WebSocket is not connected. Waiting to join channels....");
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
        println!("Current channels queue size: {}", channels.len());

        // TODO!  Joining the channel is the next step to look at.
        for channel in channels.iter_mut() {
            if channel.state == ChannelState::NotConnected {
                self.send_command(&format!("JOIN #{}", channel.name.to_lowercase()))
                    .await;
                println!("Joining channel: {}", channel.name);
                channel.state = ChannelState::Connected;
            } else {
                println!("Already joined channel: {}", channel.name);
            }
        }
    }

    pub async fn listen_for_messages(
        &self,
        read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>,
    ) {
        pin_mut!(read);
        while let Some(message) = read.next().await {
            println!("*Message RAW: {:?}", message);
            match message {
                Ok(Message::Text(text)) => {
                    if text.starts_with("PING") {
                        println!("Received PING, sending: PONG");
                        self.send_command(&text.replace("PING", "PONG")).await;
                    } else if text.contains("PRIVMSG #") {
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
