use std::sync::Mutex;
use std::{collections::{hash_map::Entry, HashMap}, sync::Arc};
use common::config::NodeInfo;
use common::ext_interface::Logger;

use common::types::U256;

use common::web_rtc::WSSignalMessage;

use common::web_rtc::WebSocketMessage;
use common::websocket::WSMessage;
use futures::executor;

use super::node_entry::NodeEntry;

pub struct Internal {
    pub logger: Box<dyn Logger>,
    pub nodes: HashMap<U256, NodeEntry>,
}

impl Internal {
    pub fn new(logger: Box<dyn Logger>) -> Arc<Mutex<Internal>> {
        let int = Arc::new(Mutex::new(Internal {
            logger,
            nodes: HashMap::new(),
        }));
        int
    }

    /// Treats incoming messages from nodes.
    pub fn cb_msg(&mut self, entry: &U256, msg: WSMessage) {
        self.logger
            .info(&format!("Got new message from {:?}: {:?}", entry, msg));
        match msg {
            WSMessage::MessageString(s) => self.receive_msg(entry, s),
            WSMessage::Closed(_) => self.close_ws(),
            WSMessage::Opened(_) => self.opened_ws(),
            WSMessage::Error(_) => self.error_ws(),
        }
    }

    fn error_ws(&self) {}
    fn close_ws(&self) {}
    fn opened_ws(&self) {}

    fn receive_msg(&mut self, entry: &U256, msg: String) {
        let msg_ws = match WebSocketMessage::from_str(&msg) {
            Ok(mw) => mw,
            Err(e) => {
                self.logger.error(&format!(
                    "Couldn't parse message as WebSocketMessage: {:?}",
                    e
                ));
                return;
            }
        };

        match msg_ws.msg {
            // Node sends his information to the server
            WSSignalMessage::Announce(msg_ann) => {
                self.logger
                    .info(&format!("Storing node {:?}", msg_ann.node_info));
                let public = msg_ann.node_info.public.clone();
                self.nodes.retain(|_, ni| {
                    if let Some(info) = ni.info.clone() {
                        return info.public != public;
                    }
                    return true;
                });
                self.nodes
                    .entry(entry.clone())
                    .and_modify(|ne| ne.info = Some(msg_ann.node_info));
                self.logger.info(&format!("Final list is {:?}", self.nodes));
            }

            // Node requests deleting of the list of all nodes
            // TODO: remove this after debugging is done
            WSSignalMessage::ClearNodes => {
                self.logger.info("Clearing nodes");
                self.nodes.clear();
            }

            // Node requests a list of all currently connected nodes,
            // including itself.
            WSSignalMessage::ListIDsRequest => {
                self.logger.info("Sending list IDs");
                let ids: Vec<NodeInfo> = self
                    .nodes
                    .iter()
                    .filter(|ne| ne.1.info.is_some())
                    .map(|ne| ne.1.info.clone().unwrap())
                    .collect();
                    self.send_message_errlog(entry, WSSignalMessage::ListIDsReply(ids));
            }

            // Node sends a PeerRequest with some of the data set to 'Some'.
            WSSignalMessage::PeerSetup(pr) => {
                self.logger.info(&format!("Got a PeerSetup {:?}", pr));
                let dst = if *entry == pr.id_init {
                    &pr.id_follow
                } else {
                    &pr.id_init
                };
                self.send_message_errlog(dst, WSSignalMessage::PeerSetup(pr.clone()));
            }
            _ => {}
        }
    }

    fn send_message_errlog(&mut self, entry: &U256, msg: WSSignalMessage){
        if let Err(e) = self.send_message(entry, msg.clone()){
            self.logger.error(&format!("Error {} while sending {:?}", e, msg));
        }
    }

    /// Tries to send a message to the indicated node.
    /// If the node is not reachable, an error will be returned.
    pub fn send_message(&mut self, entry: &U256, msg: WSSignalMessage) -> Result<(), String> {
        let msg_str = serde_json::to_string(&WebSocketMessage { msg }).unwrap();
        match self.nodes.entry(entry.clone()){
            Entry::Occupied(mut e) => {
                executor::block_on((e.get_mut().conn).send(msg_str)).unwrap();
                Ok(())
            }
            Entry::Vacant(_) => {
                self.logger.info(&format!("node {} not found", entry));
                Err("Destination not reachable".to_string())
            }
        }
    }
}
