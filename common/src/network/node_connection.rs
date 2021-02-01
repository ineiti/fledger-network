use crate::web_rtc::{
    setup::ProcessResult, PeerMessage, WebRTCConnection, WebRTCConnectionState, WebRTCMessageCB,
};
use crate::web_rtc::{setup::WebRTCSetup, WebRTCSpawner};
use std::sync::{Arc, Mutex};

/// There might be up to two connections per remote node.
/// This is in the case both nodes try to set up a connection at the same time.
/// This race condition is very difficult to catch, so it's easier to just allow
/// two connections per remote node.
/// If a second, third, or later incoming connection from the same node happens, the previous
/// connection is considered stale and discarded.
pub struct NodeConnection {
    // outgoing connections are the preferred ones.
    pub outgoing: Option<Box<dyn WebRTCConnection>>,
    // the setup connection, which will be None once the connection exists.
    pub outgoing_setup: Option<WebRTCSetup>,
    // during the setup of the connection, all new messages to be sent go into
    // the queue.
    outgoing_queue: Vec<String>,

    // incoming connections are connections initiated from another node.
    pub incoming: Option<Box<dyn WebRTCConnection>>,
    pub incoming_setup: Option<WebRTCSetup>,

    web_rtc: Arc<Mutex<WebRTCSpawner>>,
    cb_msg: Arc<Mutex<WebRTCMessageCB>>,
}

pub enum ConnectionType<'a> {
    Setup,
    Connection(&'a mut Box<dyn WebRTCConnection>),
}

impl NodeConnection {
    pub fn new(web_rtc: Arc<Mutex<WebRTCSpawner>>, cb_msg: WebRTCMessageCB) -> NodeConnection {
        NodeConnection {
            incoming: None,
            incoming_setup: None,
            outgoing: None,
            outgoing_setup: None,
            outgoing_queue: vec![],
            web_rtc,
            cb_msg: Arc::new(Mutex::new(cb_msg)),
        }
    }

    pub fn get_connection(&mut self) -> Option<ConnectionType> {
        if let Some(conn) = self.outgoing.as_mut() {
            return Some(ConnectionType::Connection(conn));
        }
        if let Some(conn) = self.incoming.as_mut() {
            return Some(ConnectionType::Connection(conn));
        }
        if self.outgoing_setup.is_some() || self.incoming_setup.is_some() {
            return Some(ConnectionType::Setup);
        }

        return None;
    }

    pub fn send(&mut self, msg: String) -> Result<(), String> {
        if let Some(ct) = self.get_connection() {
            match ct {
                ConnectionType::Setup => Ok(self.outgoing_queue.push(msg)),
                ConnectionType::Connection(conn) => conn.send(msg),
            }
        } else {
            Err("Neither connection nor setup".to_string())
        }
    }

    pub fn get_setup(&self, state: WebRTCConnectionState) -> Result<WebRTCSetup, String> {
        let conn = (self.web_rtc.lock().unwrap())(state)?;
        Ok(WebRTCSetup::new(Arc::new(Mutex::new(conn)), state))
    }

    pub async fn process_peer_setup(
        &mut self,
        pi_message: PeerMessage,
        remote: bool,
    ) -> Result<Option<PeerMessage>, String> {
        if remote {
            self.process_peer_setup_incoming(pi_message).await
        } else {
            self.process_peer_setup_outgoing(pi_message).await
        }
    }

    /// Process message for incoming webrtc setup.
    pub async fn process_peer_setup_incoming(
        &mut self,
        pi_message: PeerMessage,
    ) -> Result<Option<PeerMessage>, String> {
        match self.incoming_setup.as_mut() {
            Some(web) => match web.process(pi_message).await? {
                ProcessResult::Message(message) => {
                    return Ok(Some(message));
                }
                ProcessResult::Connection(new_conn) => {
                    self.incoming_setup = None;
                    let cb = Arc::clone(&self.cb_msg);
                    new_conn.set_cb_message(Box::new(move |msg| (cb.lock().unwrap())(msg)));
                    self.incoming = Some(new_conn);
                    return Ok(None);
                }
            },
            None => {
                if let PeerMessage::Offer(_) = pi_message {
                    let mut web = self.get_setup(WebRTCConnectionState::Follower)?;
                    match web.process(pi_message).await? {
                        ProcessResult::Message(message) => {
                            self.outgoing_setup = Some(web);
                            return Ok(Some(message));
                        }
                        _ => return Err("couldn't start webrtc handshake".to_string()),
                    }
                }
                return Err(
                    "Can only start follower webrtc handshake with Offer message".to_string(),
                );
            }
        }
    }

    /// Process message for outgoing webrtc setup.
    pub async fn process_peer_setup_outgoing(
        &mut self,
        pi_message: PeerMessage,
    ) -> Result<Option<PeerMessage>, String> {
        match self.outgoing_setup.as_mut() {
            Some(web) => match web.process(pi_message).await? {
                ProcessResult::Message(message) => {
                    return Ok(Some(message));
                }
                ProcessResult::Connection(new_conn) => {
                    self.outgoing_setup = None;
                    let cb = Arc::clone(&self.cb_msg);
                    new_conn.set_cb_message(Box::new(move |msg| (cb.lock().unwrap())(msg)));
                    self.outgoing = Some(new_conn);
                    return Ok(Some(PeerMessage::Done));
                }
            },
            None => {
                if pi_message != PeerMessage::Init {
                    return Err(
                        "Can only start outgoing webrtc setup with Init message".to_string()
                    );
                }
                let mut web = self.get_setup(WebRTCConnectionState::Initializer)?;
                match web.process(PeerMessage::Init).await? {
                    ProcessResult::Message(message) => {
                        self.outgoing = None;
                        self.outgoing_setup = Some(web);
                        return Ok(Some(message));
                    }
                    _ => return Err("couldn't start webrtc handshake".to_string()),
                }
            }
        }
    }
}
