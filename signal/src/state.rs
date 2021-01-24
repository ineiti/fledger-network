use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;

use common::config::NodeInfo;
use common::ext_interface::Logger;

use common::types::U256;


use common::web_rtc::Message;use common::web_rtc::PeerInfo;

use common::web_rtc::WebSocketMessage;use common::websocket::WSMessage;use common::websocket::WebSocketConnection;
use common::websocket::WebSocketServer;use futures::executor;

struct NodeEntry {
    conn: Box<dyn WebSocketConnection>,
    info: Option<NodeInfo>,
    peers: HashMap<U256, PeerInfo>,
}

pub struct ServerState {
    logger: Box<dyn Logger>,
    ws: Box<dyn WebSocketServer>,
    nodes: Mutex<HashMap<U256, NodeEntry>>,
}

/// This holds the logic of the signalling server.
/// It can do the following;
/// - listen for incoming websocket requests
/// - handle webrtc signalling setup
impl ServerState {
    pub fn new(logger: Box<dyn Logger>, ws: Box<dyn WebSocketServer>) -> ServerState {
        let state = ServerState {
            logger,
            ws,
            nodes: Mutex::new(HashMap::new()),
        };
        state.register_new_connection();
        state.logger.info("State started");

        state
    }

    /// Register listener for new connections.
    fn register_new_connection(&'static self) {
        self.ws
            .set_cb_connection(Box::new(move |c| self.cb_connection(c)));
    }

    /// Bogous wait for all done.
    pub fn wait_done(&self) {
        loop {
            thread::park();
        }
    }

    /// Treats new connections from websockets.
    fn cb_connection(&'static self, conn: Box<dyn WebSocketConnection>) {
        let challenge = U256::rnd();
        let ch_cl = challenge.clone();
        conn.set_cb_wsmessage(Box::new(move |cb| self.cb_msg(&ch_cl, cb)));

        let msg = serde_json::to_string(&WebSocketMessage{
            msg: Message::Challenge(challenge.clone()),
        }).unwrap();
        self.nodes.lock().unwrap().insert(
            challenge,
            NodeEntry {
                peers: HashMap::new(),
                info: None,
                conn,
            },
        );
        if let Err(e) = executor::block_on(conn.send(msg)) {
            self.logger
                .error(&format!("while sending challenge: {}", e));
        }
    }

    async fn send_message(&self, entry: &U256, msg: Message) -> Result<(), String>{
        let msg_str = serde_json::to_string(&WebSocketMessage{
            msg,
        }).unwrap();
        self.nodes.lock().unwrap().entry(entry.clone()).and_modify(|ent|
        ent.conn.send(msg_str))
    }

    /// Treats incoming messages from nodes.
    fn cb_msg(&self, entry: &U256, msg: WSMessage) {
        self.logger
            .info(&format!("Got new message from {:?}: {:?}", entry, msg));
        match msg {
            WSMessage::MessageString(s) => {self.receive_msg(entry, s)}
            WSMessage::Closed(_) => {self.close_ws()}
            WSMessage::Opened(_) => {self.opened_ws()}
        }
    }

    fn close_ws(&self){}
    fn opened_ws(&self){}

    fn receive_msg(&self, entry: &U256, msg: String){
        let msg_ws: WebSocketMessage = serde_json::from_str(&msg).unwrap();

        let mut entry_hash = self.nodes.lock().unwrap();
        match msg_ws.msg {
            // Node sends his information to the server
            Message::Announce(node) => {
                entry_hash
                    .entry(entry.clone())
                    .and_modify(|ne| ne.info = Some(node));
            }

            // Node requests deleting of the list of all nodes
            // TODO: remove this after debugging is done
            Message::ClearNodes => {
                self.nodes.lock().unwrap().clear();
            }

            // Node requests a list of all currently connected nodes,
            // including itself.
            Message::ListIDsRequest => {
                let ids: Vec<NodeInfo> = self
                    .nodes
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|ne| ne.1.info.is_some())
                    .map(|ne| ne.1.info.clone().unwrap())
                    .collect();
                (msg.send)(Message::ListIDsReply(ids));
            }

            // Node sends a PeerRequest with some of the data set to 'Some'.
            Message::PeerRequest(pr) => {
                entry_hash.entry(entry.clone()).and_modify(|ne| {
                    ne.peers.insert(pr.node.clone(), pr.clone());
                });
                let node_info = entry_hash.get(&entry.clone()).unwrap();
                if let Some(other) = entry_hash.get(&pr.node) {
                    if let Some(other_pr) =
                        other.peers.get(&node_info.info.clone().unwrap().public.clone())
                    {
                        (msg.send)(Message::PeerReply(other_pr.clone()));
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {}
