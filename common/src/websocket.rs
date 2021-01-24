use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum WSMessage {
    MessageString(String),
    Closed(String),
    Opened(String),
}

pub type MessageCallback = Box<dyn FnMut(WSMessage)>;

#[async_trait(?Send)]
pub trait WebSocketConnection {
    fn set_cb_wsmessage(&mut self, cb: MessageCallback);
    async fn send(&self, msg: String) -> Result<(), String>;
}

pub type NewConnectionCallback = Box<dyn FnMut(Box<dyn WebSocketConnection>)>;

#[async_trait(?Send)]
pub trait WebSocketServer {
    fn set_cb_connection(&mut self, cb: NewConnectionCallback);
}
