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
    async fn send(&mut self, msg: String) -> Result<(), String>;
}

//
// These definitions are only for the unix-binaries, where
// threads are available.
//

pub type MessageCallbackSend = Box<dyn FnMut(WSMessage) + Send>;

#[async_trait]
pub trait WebSocketConnectionSend: Send {
    fn set_cb_wsmessage(&mut self, cb: MessageCallbackSend);
    async fn send(&mut self, msg: String) -> Result<(), String>;
}

pub type NewConnectionCallback = Box<dyn FnMut(Box<dyn WebSocketConnectionSend + Send>) + Send>;

pub trait WebSocketServer {
    fn set_cb_connection(&mut self, cb: NewConnectionCallback);
}
