use common::{
    config::NodeInfo,
    ext_interface::Logger,
    types::U256,
    web_rtc::{Message, PeerInfo, WebSocketMessage},
    websocket::WebSocketConnectionSend,
};

use futures::executor;
use std::collections::HashMap;

pub struct NodeEntry {
    pub conn: Box<dyn WebSocketConnectionSend>,
    pub info: Option<NodeInfo>,
    pub peers: HashMap<U256, PeerInfo>,
    entry: U256,
    logger: Box<dyn Logger>,
}

impl NodeEntry {
    pub fn new(
        logger: Box<dyn Logger>,
        entry: U256,
        conn: Box<dyn WebSocketConnectionSend>,
    ) -> NodeEntry {
        let mut ne = NodeEntry {
            info: None,
            logger,
            entry,
            peers: HashMap::new(),
            conn,
        };
        let msg = serde_json::to_string(&WebSocketMessage {
            msg: Message::Challenge(ne.entry.clone()),
        })
        .unwrap();
        if let Err(e) = executor::block_on(ne.conn.send(msg)) {
            ne.logger.error(&format!("while sending challenge: {}", e));
        }
        ne
    }
}
