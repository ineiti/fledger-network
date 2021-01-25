use crate::{
    config::NodeInfo,
    ext_interface::{WebRTCConnection, WebRTCSpawner},
    web_rtc::MessageAnnounce,
};
use crate::{
    ext_interface::Logger,
    web_rtc::{Message, WebSocketMessage},
};

use crate::types::U256;
use crate::websocket::WSMessage;
use crate::websocket::WebSocketConnection;
use std::{pin::Pin, sync::Mutex};
use std::{collections::HashMap, sync::Arc};

pub struct Network {
    intern: Arc<Mutex<Intern>>,
}

/// There might be up to two connections per remote node.
/// This is in the case both nodes try to set up a connection at the same time.
/// This race condition is very difficult to catch, so it's easier to just allow
/// two connections per remote node.
struct NodeConnection {
    conn: Vec<Box<dyn WebRTCConnection>>,
}

struct Intern {
    ws: Box<dyn WebSocketConnection>,
    web_rtc: WebRTCSpawner,
    connections: HashMap<U256, NodeConnection>,
    logger: Box<dyn Logger>,
    node_info: Option<NodeInfo>,
    pub list: Vec<NodeInfo>,
}

impl Intern {
    pub fn msg_cb(&mut self, msg: WSMessage) {
        match msg {
            WSMessage::MessageString(s) => {
                self.logger.info(&format!("Got a MessageString: {:?}", s));
                match WebSocketMessage::from_str(&s) {
                    Ok(wsm) => self.process_msg(wsm.msg),
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

    fn process_msg(&mut self, msg: Message) {
        match msg {
            Message::Challenge(challenge) => {
                let ma = MessageAnnounce {
                    challenge,
                    node_info: self.node_info.clone().unwrap(),
                };
                self.send(Message::Announce(ma));
            }
            Message::ListIDsReply(list) => {
                self.update_list(list);
            }
            Message::PeerReply(_) => {}
            Message::Done => {}
            _ => {}
        }
    }

    pub fn send(&mut self, msg: Message) {
        self.logger
            .info(&format!("Sending {:?} over websocket", msg));
        let wsm = WebSocketMessage { msg };
        if let Err(e) = self.ws.send(wsm.to_string()) {
            self.logger.error(&format!("Error while sending: {:?}", e));
        }
    }

    pub fn set_node_info(&mut self, ni: NodeInfo) {
        self.node_info.replace(ni);
    }

    pub fn update_node_list(&mut self) {
        self.send(Message::ListIDsRequest);
    }

    pub fn update_list(&mut self, list: Vec<NodeInfo>) {
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
            intern: Arc::new(Mutex::new(Intern {
                ws,
                web_rtc,
                connections: HashMap::new(),
                logger,
                node_info: None,
                list: vec![],
            })),
        };
        net
    }

    /// Sending strings to other nodes. If the connection already exists,
    /// it will  be used to send the string over.
    /// Else the signalling server will be contacted, a webrtc connection will
    /// be created, and then the message will be sent over.
    pub fn send(&self, _dst: U256, _msg: String) -> Result<(), String> {
        Ok(())
    }

    pub fn set_node_info(&self, ni: NodeInfo) {
        Arc::clone(&self.intern).lock().unwrap().set_node_info(ni);
        let n = Arc::clone(&self.intern);
        self.intern
            .lock()
            .unwrap()
            .ws
            .set_cb_wsmessage(Box::new(move |msg| n.lock().unwrap().msg_cb(msg)));
    }

    pub fn update_node_list(&self) {
        Arc::clone(&self.intern).lock().unwrap().update_node_list();
    }

    pub fn get_list(&self) -> Vec<NodeInfo> {
        Arc::clone(&self.intern).lock().unwrap().list.clone()
    }
}
