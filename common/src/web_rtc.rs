use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{config::NodeInfo, types::U256};

pub mod setup;

pub type WebRTCSpawner =
    Box<dyn Fn(WebRTCConnectionState) -> Result<Box<dyn WebRTCConnectionSetup>, String>>;

#[async_trait(?Send)]
pub trait WebRTCConnectionSetup {
    /// Returns the offer string that needs to be sent to the `Follower` node.
    async fn make_offer(&mut self) -> Result<String, String>;

    /// Takes the offer string
    async fn make_answer(&mut self, offer: String) -> Result<String, String>;

    /// Takes the answer string and finalizes the first part of the connection.
    async fn use_answer(&mut self, answer: String) -> Result<(), String>;

    /// Waits for the ICE to move on from the 'New' state
    async fn wait_gathering(&mut self) -> Result<(), String>;

    /// Waits for the ICE string to be avaialble.
    async fn ice_string(&mut self) -> Result<String, String>;

    /// Sends the ICE string to the WebRTC.
    async fn ice_put(&mut self, ice: String) -> Result<(), String>;

    /// Debugging output of the RTC state
    async fn print_states(&mut self);

    /// Returns the connection. This only works once all correct calls
    /// have been made and the webrtc is correctly set up.
    async fn get_connection(&mut self) -> Result<Box<dyn WebRTCConnection>, String>;
}

pub trait WebRTCConnection {
    /// Send a message to the other node. This call blocks until the message
    /// is queued.
    fn send(&self, s: String) -> Result<(), String>;

    /// Sets the callback for incoming messages.
    fn set_cb_message(&self, cb: WebRTCMessageCB);
}

pub type WebRTCMessageCB = Box<dyn FnMut(String)>;

/// What type of node this is
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WebRTCConnectionState {
    Initializer,
    Follower,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PeerMessage {
    Init,
    Offer(String),
    Answer(String),
    IceInit(String),
    IceFollow(String),
    Done,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PeerInfo {
    pub id_init: U256,
    pub id_follow: U256,
    pub message: PeerMessage,
}

impl PeerInfo {
    pub fn new(init: &U256, follow: &U256) -> PeerInfo {
        PeerInfo {
            id_init: init.clone(),
            id_follow: follow.clone(),
            message: PeerMessage::Init,
        }
    }

    pub fn get_remote(&self, local: &U256) -> Option<U256> {
        if self.id_init == local.clone() {
            return Some(self.id_follow.clone());
        }
        if self.id_follow == local.clone() {
            return Some(self.id_init.clone());
        }
        return None;
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSocketMessage {
    pub msg: WSSignalMessage,
}

impl WebSocketMessage {
    pub fn from_str(s: &str) -> Result<WebSocketMessage, String> {
        serde_json::from_str(s).map_err(|err| err.to_string())
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// Message is a list of messages to be sent between the node and the signal server.
/// When a new node connects to the signalling server, the server starts by sending
/// a "Challenge" to the node.
/// The node can then announce itself using that challenge.
/// - ListIDs* are used by the nodes to get a list of currently connected nodes
/// - ClearNodes is a debugging message that will be removed at a later stage.
/// - PeerRequest is sent by a node to ask to connect to another node. The
/// server will send a 'PeerReply' to the corresponding node, which will continue
/// the protocol by sending its own PeerRequest.
/// - Done is a standard message that can be sent back to indicate all is well.
///
/// TODO: use the "Challenge" to sign with the private key of the node, so that the server
/// can verify that the node knows the corresponding private key of its public key.
#[derive(Debug, Deserialize, Serialize)]
pub enum WSSignalMessage {
    Challenge(U256),
    Announce(MessageAnnounce),
    ListIDsRequest,
    ListIDsReply(Vec<NodeInfo>),
    ClearNodes,
    PeerSetup(PeerInfo),
    Done,
}

/// TODO: add a signature on the challenge
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageAnnounce {
    pub challenge: U256,
    pub node_info: NodeInfo,
}
