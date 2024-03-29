use super::client::{WebSocket, WebSocketState};
pub struct Replier {
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
}
