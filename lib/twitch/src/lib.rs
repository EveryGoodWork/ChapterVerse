use futures_util::{stream::StreamExt, SinkExt};
use rand::Rng;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

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

pub async fn connect(
    websocket_state: Arc<Mutex<WebSocketState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://irc-ws.chat.twitch.tv:80";
    println!("Attempting to connect to WebSocket at URL: {}", url);

    {
        let mut ws_state = websocket_state.lock().await;
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
        .0; // Directly take the WebSocketStream from the result tuple

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Send a test message
    let nick_command = format!(
        "NICK justinfan{}",
        rand::thread_rng().gen_range(10000..=99999)
    );
    println!("Sending NICK command: {}", &nick_command);
    if let Err(e) = tx.send(Message::Text(nick_command)) {
        println!("Failed to send NICK command: {:?}", e);
    } else {
        println!("NICK command sent successfully.");
    }

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match write.send(message).await {
                Ok(_) => println!("Message sent successfully."),
                Err(e) => println!("Error sending message: {:?}", e),
            }
        }
    });

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received message: {}", text);
                handle_message(text, &tx).await;
            }
            Ok(Message::Ping(ping)) => {
                println!("Received ping, sending pong.");
                if let Err(e) = tx.send(Message::Pong(ping)) {
                    println!("Failed to send PONG response: {:?}", e);
                }
            }
            Ok(_) => (),
            Err(e) => println!("Error reading message: {:?}", e),
        }
    }

    Ok(())
}

async fn handle_message(message: String, tx: &mpsc::UnboundedSender<Message>) {
    if message.starts_with("PING") {
        println!("Handling PING message.");
        if let Err(e) = tx.send(Message::Text("PONG".into())) {
            println!("Failed to respond to PING with PONG: {:?}", e);
        }
    } else if message.contains("PRIVMSG") {
        println!("Received PRIVMSG: {}", message);
    } else if message.contains(":Welcome, GLHF!") {
        println!("Connected to Twitch");
    }
}
