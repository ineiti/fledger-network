use common::{
    config::NodeInfo,
    ext_interface::Logger,
    types::U256,
    web_rtc::{WSSignalMessage, WebSocketMessage},
    websocket::WebSocketConnectionSend,
};

use futures::executor;
use std::{fmt};

pub struct NodeEntry {
    pub conn: Box<dyn WebSocketConnectionSend>,
    pub info: Option<NodeInfo>,
    entry: U256,
    logger: Box<dyn Logger>,
}

impl fmt::Debug for NodeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("NodeEntry: {:?}", self.info))?;
        Ok(())
    }
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
            conn,
        };
        let msg = serde_json::to_string(&WebSocketMessage {
            msg: WSSignalMessage::Challenge(ne.entry.clone()),
        })
        .unwrap();
        if let Err(e) = executor::block_on(ne.conn.send(msg)) {
            ne.logger.error(&format!("while sending challenge: {}", e));
        }
        ne
    }
}
