use log::{info, error};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::{UnboundedSender}};
use tokio::time::{self, Duration};
use crate::chat::client::WebSocketState;
use crate::chat::Listener;
use crate::common::message_data::MessageData;

pub struct Listeners {
    sender: UnboundedSender<MessageData>,
    listener_collection: Arc<Mutex<Vec<Arc<Listener>>>>,
}

const MAX_CHANNELS: usize = 2;

impl Listeners {
    pub fn new(sender: UnboundedSender<MessageData>) -> Arc<Self> {
        let manager = Arc::new(Self {
            sender,
            listener_collection: Arc::new(Mutex::new(Vec::new())),
        });
        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            manager_clone.monitor_listeners_loop().await;
        });
        manager
    }

    pub async fn add_channel(&self, channel: &str) {
        let listener = {
            let mut listeners = self.listener_collection.lock().await;
            listeners.iter()
                .find(|listener| {
                    let count = listener.channels_count.load(std::sync::atomic::Ordering::SeqCst);
                    count < MAX_CHANNELS
                })
                .map(Arc::clone)
                .or_else(|| {
                    let new_listener = Arc::new(Listener::new(self.sender.clone(), None, None));
                    listeners.push(Arc::clone(&new_listener));
                    Some(new_listener)
                })
        };
        let _ = listener.clone().unwrap().join_channel(&channel).await;
        info!("Channel added: {}", channel);
    }

    async fn monitor_listeners_loop(&self) {
        loop {
            {
                let listeners = self.listener_collection.lock().await;
                for listener in listeners.iter() {
                    let count = listener.channels_count.load(std::sync::atomic::Ordering::SeqCst);
                    let state = listener.get_state();
                    info!("Listener state: {:?}, Channels joined: {}", state, count);
                    if state != WebSocketState::Connected {
                        let result = <Arc<Listener> as Clone>::clone(&listener).connect().await;
                        if let Err(e) = result {
                            error!("Failed to connect: {:?}", e);
                        }
                    }
                }
            }
            time::sleep(Duration::from_secs(5)).await;
        }
    }
}