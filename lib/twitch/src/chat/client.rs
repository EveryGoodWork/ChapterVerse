use crate::common;
use crate::common::channel_data::{Channel, ChannelState};
use crate::common::message_data::{MessageData, Type};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify, RwLock};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use futures_util::stream::SplitStream;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
const BUCKET_CAPACITY: usize = 100;
const LEAK_RATE: Duration = Duration::from_millis(1500);

struct JoinRateLimiter {
    join_attempts: Mutex<Vec<Instant>>,
    max_attempts: usize,
    period: Duration,
}

impl JoinRateLimiter {
    /// https://dev.twitch.tv/docs/irc/#rate-limits
    /// https://discuss.dev.twitch.com/t/giving-broadcasters-control-concurrent-join-limits-for-irc-and-eventsub/54997
    /// 20 join attempts per 10 seconds per user.
    // TODO!  This is going to be a problem.  Concurrent channel join limit reduced to 100.
    pub fn new(max_attempts: usize, period: Duration) -> Self {
        JoinRateLimiter {
            join_attempts: Mutex::new(Vec::new()),
            max_attempts,
            period,
        }
    }

    pub async fn wait_and_record(&self) {
        loop {
            let now = Instant::now();
            let mut attempts = self.join_attempts.lock().await;
            attempts.retain(|&x| x.elapsed() < self.period);

            if attempts.len() < self.max_attempts {
                attempts.push(now);
                break;
            } else {
                if let Some(first) = attempts.first() {
                    let wait_until = *first + self.period;
                    let now = Instant::now();
                    if wait_until > now {
                        drop(attempts);
                        tokio::time::sleep(wait_until - now).await;
                    } else {
                        attempts.push(now);
                        break;
                    }
                }
            }
        }
    }
}

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
    twitch_stream:
        Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WebSocketMessage>>>>,
    twitch_channel_transmitter: Arc<Mutex<Option<UnboundedSender<WebSocketMessage>>>>,
    message_bucket: Arc<RwLock<VecDeque<String>>>,
    message_bucket_notifier: Arc<Notify>,
    channel_timeouts: Arc<RwLock<HashMap<String, Instant>>>,
    join_rate_limiter: Arc<JoinRateLimiter>,
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
            twitch_stream: Arc::new(Mutex::new(None)),
            twitch_channel_transmitter: Arc::new(Mutex::new(None)),
            channels: Arc::new(Mutex::new(VecDeque::new())),
            message_bucket: Arc::new(RwLock::new(VecDeque::new())),
            message_bucket_notifier: Arc::new(Notify::new()),
            channel_timeouts: Arc::new(RwLock::new(HashMap::new())),
            join_rate_limiter: Arc::new(JoinRateLimiter::new(20, Duration::from_secs(10))),
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
                    // println!("Attempting to connect...");
                    self.set_state(WebSocketState::Connecting).await;
                }
                WebSocketState::Connecting | WebSocketState::Connected => {
                    // println!("Already connecting or connected, no action taken.");
                    return Ok(());
                }
                WebSocketState::Failed => {
                    eprintln!("Previous attempt failed, trying again...");
                }
            }
            let url = "ws://irc-ws.chat.twitch.tv:80";
            match connect_async(url).await {
                Ok((ws_stream, _)) => {
                    self.handle_connection_success(ws_stream).await;
                    // println!("---WebSocket Connected and ready.");
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
    async fn handle_connection_success(self: Arc<Self>, ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) {
        let (mut split_sink, split_stream) = ws_stream.split();
        let (twitch_transmitter, twitch_receiver) = mpsc::unbounded_channel();

        // Spawn the task for listening to messages
        tokio::spawn(Self::handle_twitch_receive(self.clone(), split_stream));

        if let Some(oauth) = &self.oauth_token {
            if let Err(e) = split_sink.send(format!("PASS {}", oauth).into()).await {
                eprintln!("Error sending message: {:?}", e);
                self.set_state(WebSocketState::Disconnected).await;
            }
        }

        if let Err(e) = split_sink.send(format!("NICK {}", self.username).into()).await {
            eprintln!("Error sending message: {:?}", e);
            self.set_state(WebSocketState::Disconnected).await;
        }

        if let Err(e) = split_sink.send("CAP REQ :twitch.tv/tags twitch.tv/commands".into()).await {
            eprintln!("Error sending message: {:?}", e);
            self.set_state(WebSocketState::Disconnected).await;
        }


        // Spawn the send task with the twitch_receiver before any other operations that might move it
        tokio::spawn(Self::handle_twitch_send(self.clone(), twitch_receiver));

        // Store the split_sink and twitch_transmitter into their respective locks
        {
            let mut sink_lock = self.twitch_stream.lock().await;
            *sink_lock = Some(split_sink);
        }
        {
            let mut transmitter_lock = self.twitch_channel_transmitter.lock().await;
            *transmitter_lock = Some(twitch_transmitter);
        }

        // Start the leaky bucket handler
        self.clone().start_leaky_bucket_handler();
    }

    async fn handle_twitch_send(self: Arc<Self>, mut receiver: UnboundedReceiver<WebSocketMessage>) {
        while let Some(message) = receiver.recv().await {
            let mut sink = self.twitch_stream.lock().await;
            if let Some(sink) = sink.as_mut() {
                if let Err(e) = sink.send(message).await {
                    eprintln!("Error sending message: {:?}", e);
                    self.set_state(WebSocketState::Disconnected).await;
                }
            }
        }
    }

    async fn handle_twitch_receive(self: Arc<Self>, mut split_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
        while let Some(message_result) = split_stream.next().await {
            match message_result {
                Ok(message) => {
                    // println!("RAW Message {:?}",message);
                    if let Message::Text(text) = message {
                        if text.starts_with("PING") {
                            self.send_command(&text.replace("PING", "PONG")).await;
                        } else if text.contains("PRIVMSG #") {
                            if let Some(parsed_message) = common::message_data::MessageData::new(&text)
                            {
                                if let Some(sender) = &self.message_tx {
                                    if let Err(e) = sender.send(parsed_message) {
                                        eprintln!("PRIVMSG: Failed to send message to channel: {}", e);
                                    }
                                }
                            }
                        } else if !text.contains("PRIVMSG") & text.contains(":Welcome, GLHF!") {
                            self.set_state(WebSocketState::Connected).await;
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error receiving message: {:?}", e);
                    self.set_state(WebSocketState::Disconnected).await;
                }
            }
        }
    }

    pub async fn send_message(&self, message: MessageData) {
        if message.tags.contains(&Type::PRIVMSG) {
            let twitch_message = match message.reply {
                Some(ref reply) => match message.id {
                    Some(id) => format!(
                        "@reply-parent-msg-id={} PRIVMSG #{} :{}\r\n",
                        id, message.channel, reply
                    ),
                    None => format!("PRIVMSG #{} :{}\r\n", message.channel, reply),
                },
                None => format!("PRIVMSG #{} :{}\r\n", message.channel, message.text),
            };

            let mut message_bucket = self.message_bucket.write().await;
            if message_bucket.len() < BUCKET_CAPACITY {
                message_bucket.push_back(twitch_message.clone());
                self.message_bucket_notifier.notify_one();
            } else {
                // println!("Bucket full, message throttled: {}", twitch_message);
            }
        } else if message.tags.contains(&Type::WHISPER) {
            let whisper_message = format!(
                "WHISPER {} :{}\r\n",
                message.display_name.unwrap().to_lowercase(),
                message.reply.unwrap()
            );
            // println!("Whisper Message: {}", whisper_message);
            let mut message_bucket = self.message_bucket.write().await;
            message_bucket.push_back(whisper_message.clone());
            self.message_bucket_notifier.notify_one();
        }
    }

    pub fn start_leaky_bucket_handler(self: Arc<Self>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            loop {
                self_clone.message_bucket_notifier.notified().await;

                let mut skipped_messages = VecDeque::new();
                let mut processed_any = true;

                while processed_any {
                    processed_any = false;

                    {
                        let now = Instant::now();
                        let mut message_bucket = self_clone.message_bucket.write().await;
                        let mut channel_timeouts = self_clone.channel_timeouts.write().await;

                        while let Some(message) = message_bucket.pop_front() {
                            processed_any = true;

                            if let Some(channel_name) = Self::extract_channel_name(&message) {
                                if let Some(&timeout) = channel_timeouts.get(&channel_name) {
                                    if now < timeout {
                                        skipped_messages.push_back(message);
                                        continue;
                                    }
                                }

                                channel_timeouts.insert(channel_name.clone(), now + LEAK_RATE);

                                if let Some(transmitter) =
                                    &*self_clone.twitch_channel_transmitter.lock().await
                                {
                                    transmitter.send(message.into()).unwrap_or_else(|e| {
                                        eprintln!("Error sending message: {:?}", e);
                                        let _ = self.set_state(WebSocketState::Disconnected);
                                    });
                                }
                                tokio::time::sleep(LEAK_RATE).await;
                            }
                        }
                    }

                    if processed_any {
                        let mut message_bucket = self_clone.message_bucket.write().await;
                        message_bucket.extend(skipped_messages.drain(..));
                    }
                }

                if !processed_any && skipped_messages.is_empty() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
    }

    fn extract_channel_name(message: &str) -> Option<String> {
        let (_before, after) = message.split_once("PRIVMSG #")?;
        after
            .split_once(' ')
            .map(|(channel, _)| channel.to_string())
    }

    pub async fn send_command(&self, command_text: &str) {
        // println!("send_command {}", command_text);
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
                // println!("Channel already exists: {}", channel_name);
            }
            // println!("join_channel channels queue size: {}", channels.len());
        }
        self.join_pending_channels().await;
    }

    pub async fn leave_channel(self: Arc<Self>, channel_name: &str) {
        self.send_command(&format!("PART #{}", channel_name.to_lowercase()))
            .await;
    }

    pub async fn join_pending_channels(self: Arc<Self>) {

        if self.get_state() == WebSocketState::Connected {
            self.process_channel_joining().await;
        } else {
            // println!("WebSocket is not connected. Waiting to join channels....");
            tokio::spawn({
                let ws_clone = Arc::clone(&self);
                async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        if ws_clone.get_state() == WebSocketState::Connected {
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
        // println!("Current channels queue size: {}", channels.len());
        for channel in channels.iter_mut() {
            if channel.state == ChannelState::NotConnected {
                self.join_rate_limiter.wait_and_record().await;
                self.send_command(&format!("JOIN #{}", channel.name.to_lowercase()))
                    .await;
                // println!(
                //     "Joining channel: {} {:?}",
                //     channel.name,
                //     time::SystemTime::now()
                // );
                channel.state = ChannelState::Connected;
            } else {
                // println!("Already joined channel: {}", channel.name);
            }
        }
    }

    // pub async fn listen_for_messages(
    //     self: Arc<Self>,
    //     read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>,
    // ) {
    //     pin_mut!(read);
    //     let self_clone = Arc::clone(&self);
    //     while let Some(message) = read.next().await {
    //         // println!("*Message RAW: {:?}", message);
    //         match message {
    //             Ok(Message::Text(ref text)) => {
    //                 if message.is_ok() {
    //                     if text.starts_with("PING") {
    //                         //println!("DEBUG Received PING, sending: PONG");
    //                         self_clone.send_command(&text.replace("PING", "PONG")).await;
    //                     } else if text.contains("PRIVMSG #") {
    //                         if let Some(parsed_message) = common::message_data::MessageData::new(&text)
    //                         {
    //                             if let Some(sender) = &self_clone.message_tx {
    //                                 if let Err(e) = sender.send(parsed_message) {
    //                                     eprintln!("PRIVMSG: Failed to send message to channel: {}", e);
    //                                 }
    //                             }
    //                         }
    //                     } else if text.contains("WHISPER") {
    //                         if let Some(parsed_message) = common::message_data::MessageData::new(&text)
    //                         {
    //                             if let Some(sender) = &self_clone.message_tx {
    //                                 if let Err(e) = sender.send(parsed_message) {
    //                                     eprintln!("WHISPER: Failed to send message to channel: {}", e);
    //                                 }
    //                             }
    //                         }
    //                     } else if !text.contains("PRIVMSG") & text.contains(":Welcome, GLHF!") {
    //                         // println!("{} {:?}", ":Welcome, GLHF! ", self.username);
    //                         self_clone.set_state(WebSocketState::Connected).await;
    //                     }
    //                 }
    //                 else {
    //                     eprintln!("Message ERROR: {:?}", message);
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Error receiving message: {:?}", e);
    //                 if let tokio_tungstenite::tungstenite::Error::Io(io_error) = &e {
    //                     if io_error.kind() == std::io::ErrorKind::ConnectionReset {
    //                         eprintln!("Connection was reset by the remote host.");
    //                         self_clone.clone().set_state(WebSocketState::Disconnected).await;
    //                         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //                         // self_clone.clone().connect().await;
    //                     }
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    // }
}
