use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use common::config::NodeInfo;
use common::ext_interface::Logger;

use common::types::U256;

use common::web_rtc::Message;

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
        }
    }

    fn close_ws(&self) {}
    fn opened_ws(&self) {}

    fn receive_msg(&mut self, entry: &U256, msg: String) {
        let msg_ws: WebSocketMessage = serde_json::from_str(&msg).unwrap();

        match msg_ws.msg {
            // Node sends his information to the server
            Message::Announce(node) => {
                self.nodes
                    .entry(entry.clone())
                    .and_modify(|ne| ne.info = Some(node));
            }

            // Node requests deleting of the list of all nodes
            // TODO: remove this after debugging is done
            Message::ClearNodes => {
                self.nodes.clear();
            }

            // Node requests a list of all currently connected nodes,
            // including itself.
            Message::ListIDsRequest => {
                let ids: Vec<NodeInfo> = self
                    .nodes
                    .iter()
                    .filter(|ne| ne.1.info.is_some())
                    .map(|ne| ne.1.info.clone().unwrap())
                    .collect();
                let msg_str = serde_json::to_string(&Message::ListIDsReply(ids)).unwrap();
                self.nodes
                    .entry(entry.clone())
                    .and_modify(|ne| executor::block_on(ne.conn.send(msg_str)).unwrap());
            }

            // Node sends a PeerRequest with some of the data set to 'Some'.
            Message::PeerRequest(pr) => {
                self.nodes.entry(entry.clone()).and_modify(|ne| {
                    ne.peers.insert(pr.node.clone(), pr.clone());
                });
                let node_info = self.nodes.get(&entry.clone()).unwrap();
                let mut msg: Option<Message> = None;
                if let Some(other) = self.nodes.get(&pr.node) {
                    if let Some(other_pr) = other
                        .peers
                        .get(&node_info.info.clone().unwrap().public.clone())
                    {
                        msg = Some(Message::PeerReply(other_pr.clone()));
                    }
                }
                if msg.is_some() {
                    self.send_message(entry, msg.unwrap());
                }
            }
            _ => {}
        }
    }

    /// TODO: should the error be caught somewhere?
    pub fn send_message(&mut self, entry: &U256, msg: Message) {
        let msg_str = serde_json::to_string(&WebSocketMessage { msg }).unwrap();
        self.nodes
            .entry(entry.clone())
            .and_modify(|ent| executor::block_on((ent.conn).send(msg_str)).unwrap());
    }
}
