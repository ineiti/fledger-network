use crate::{
    config::NodeInfo,
    web_rtc::{MessageAnnounce, PeerInfo, PeerMessage, WebRTCSpawner},
};
use crate::{
    ext_interface::Logger,
    web_rtc::{WSSignalMessage, WebSocketMessage},
};
use futures::executor;

use crate::types::U256;
use crate::websocket::WSMessage;
use crate::websocket::WebSocketConnection;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

pub struct Network {
    intern: Arc<Mutex<Intern>>,
}

mod node_connection;

use node_connection::NodeConnection;

struct Intern {
    ws: Box<dyn WebSocketConnection>,
    web_rtc: Arc<Mutex<WebRTCSpawner>>,
    connections: HashMap<U256, NodeConnection>,
    logger: Box<dyn Logger>,
    node_info: Option<NodeInfo>,
    pub list: Vec<NodeInfo>,
    challenge_queue: Option<U256>,
}

impl Intern {
    /// Returns a new Arc<Mutex<Intern>> wired up to process incoming messages
    /// through the WebSocket.
    pub fn new(
        ws: Box<dyn WebSocketConnection>,
        web_rtc: WebRTCSpawner,
        logger: Box<dyn Logger>,
    ) -> Arc<Mutex<Intern>> {
        let int = Arc::new(Mutex::new(Intern {
            ws,
            web_rtc: Arc::new(Mutex::new(web_rtc)),
            connections: HashMap::new(),
            logger,
            node_info: None,
            list: vec![],
            challenge_queue: None,
        }));
        let int_cl = Arc::clone(&int);
        int.lock()
            .unwrap()
            .ws
            .set_cb_wsmessage(Box::new(move |msg| int_cl.lock().unwrap().msg_cb(msg)));
        int
    }

    fn msg_cb(&mut self, msg: WSMessage) {
        match msg {
            WSMessage::MessageString(s) => {
                self.logger.info(&format!("Got a MessageString: {:?}", s));
                match WebSocketMessage::from_str(&s) {
                    Ok(wsm) => {
                        if let Err(err) = executor::block_on(self.process_msg(wsm.msg)) {
                            self.logger
                                .error(&format!("Couldn't process message: {}", err))
                        }
                    }
                    Err(err) => self
                        .logger
                        .error(&format!("While parsing message: {:?}", err)),
                }
            }
            WSMessage::Closed(_) => {}
            WSMessage::Opened(_) => {}
            WSMessage::Error(_) => {}
        }
    }

    fn public(&self) -> U256 {
        self.node_info.as_ref().unwrap().public.clone()
    }

    /// Processes incoming messages from the signalling server.
    /// This can be either messages requested by this node, or connection
    /// setup requests from another node.
    async fn process_msg(&mut self, msg: WSSignalMessage) -> Result<(), String> {
        match msg {
            WSSignalMessage::Challenge(challenge) => {
                if self.node_info.is_some() {
                    self.send_announce(challenge);
                } else {
                    self.challenge_queue = Some(challenge);
                }
            }
            WSSignalMessage::ListIDsReply(list) => {
                self.update_list(list);
            }
            WSSignalMessage::PeerSetup(pi) => {
                let remote = match pi.get_remote(&self.public()) {
                    Some(id) => id,
                    None => {
                        return Err("Got alien PeerSetup".to_string());
                    }
                };
                let conn = self
                    .connections
                    .entry(remote.clone())
                    .or_insert(NodeConnection::new(
                        Arc::clone(&self.web_rtc),
                        Box::new(move |msg| self.rcv(&remote, msg)),
                    ));

                if let Some(message) = conn
                    .process_peer_setup(pi.message, remote == pi.id_init)
                    .await?
                {
                    self.send_ws(WSSignalMessage::PeerSetup(PeerInfo { message, ..pi }));
                }
            }
            WSSignalMessage::Done => {}
            _ => {}
        }
        Ok(())
    }

    /// Sends the announcement.
    /// Because of timing issues with node_info and the Announce message,
    /// there are two code-paths up to here...
    fn send_announce(&mut self, challenge: U256) {
        let ma = MessageAnnounce {
            challenge,
            node_info: self.node_info.clone().unwrap(),
        };
        self.send_ws(WSSignalMessage::Announce(ma));
    }

    /// Sends a websocket message to the signalling server.
    /// This is not a public method, as all communication should happen using
    /// webrtc connections.
    fn send_ws(&mut self, msg: WSSignalMessage) {
        self.logger
            .info(&format!("Sending {:?} over websocket", msg));
        let wsm = WebSocketMessage { msg };
        if let Err(e) = self.ws.send(wsm.to_string()) {
            self.logger.error(&format!("Error while sending: {:?}", e));
        }
    }

    /// Updates the node info in the Internal structure.
    pub fn set_node_info(&mut self, ni: NodeInfo) {
        self.node_info.replace(ni);
        if let Some(challenge) = self.challenge_queue.clone() {
            self.send_announce(challenge);
        }
    }

    /// Requests a new node list from the server.
    pub fn update_node_list(&mut self) {
        self.send_ws(WSSignalMessage::ListIDsRequest);
    }

    /// Stores a node list sent from the signalling server.
    fn update_list(&mut self, list: Vec<NodeInfo>) {
        self.logger.info(&format!("Got new list: {:?}", list));
        let public = self.node_info.clone().unwrap().public;
        self.list = list
            .iter()
            .filter(|entry| entry.public != public)
            .cloned()
            .collect();
        self.logger
            .info(&format!("Reduced list is: {:?}", self.list));
    }

    /// Sends a message to the node dst.
    /// If no connection is setup, the msg will be put in a queue, and
    /// the connection will be setup.
    /// If the connection is in the setup phase, the msg will be put in a queue,
    /// and the method returns.
    /// All messages in the queue will be sent once the connection is set up.
    pub async fn send(&mut self, dst: &U256, msg: String) -> Result<(), String> {
        let conn = self
            .connections
            .entry(dst.clone())
            .or_insert(NodeConnection::new(
                Arc::clone(&self.web_rtc),
                Box::new(|msg| self.rcv(dst, msg)),
            ));

        if conn.send(msg.clone()).is_err() {
            conn.process_peer_setup_outgoing(PeerMessage::Init).await?;
            conn.send(msg)?;
        }
        Ok(())
    }

    pub fn rcv(&mut self, from: &U256, msg: String) {}
}

/// Network combines a websocket to connect to the signal server with
/// a WebRTC trait to connect to other nodes.
/// It supports setting up automatic connetions to other nodes.
impl Network {
    pub fn new(
        ws: Box<dyn WebSocketConnection>,
        web_rtc: WebRTCSpawner,
        logger: Box<dyn Logger>,
    ) -> Network {
        let net = Network {
            intern: Intern::new(ws, web_rtc, logger),
        };
        net
    }

    /// Sending strings to other nodes. If the connection already exists,
    /// it will be used to send the string over.
    /// Else the signalling server will be contacted, a webrtc connection will
    /// be created, and then the message will be sent over.
    /// During the setup of a new connection, the message is stored in a queue.
    /// So in the case of a new connection, the 'send' method returns even before the
    /// message is actually sent
    pub async fn send(&self, dst: &U256, msg: String) -> Result<(), String> {
        let mut int = self.intern.lock().unwrap();
        int.send(dst, msg).await
    }

    pub async fn rcv(&self, src: &U256, msg: String) {}

    pub fn set_node_info(&self, ni: NodeInfo) {
        Arc::clone(&self.intern).lock().unwrap().set_node_info(ni);
    }

    pub fn clear_nodes(&self) {
        self.intern
            .lock()
            .unwrap()
            .send_ws(WSSignalMessage::ClearNodes);
    }

    pub fn update_node_list(&self) {
        Arc::clone(&self.intern).lock().unwrap().update_node_list();
    }

    pub fn get_list(&self) -> Vec<NodeInfo> {
        Arc::clone(&self.intern).lock().unwrap().list.clone()
    }
}
