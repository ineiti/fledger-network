use crate::ext_interface::WebRTCCaller;
use crate::ext_interface::WebRTCMethod;

use serde::{Deserialize, Serialize};

use crate::{config::NodeInfo, types::U256};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PeerInfo {
    pub node: U256,
    pub offer: Option<String>,
    pub candidate: Option<String>,
    pub answer: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSocketMessage {
    pub msg: Message,
}

/// Message is a list of messages to be sent between the node and the signal server.
/// When a new node connects to the signalling server, the server starts by sending
/// a "Challenge" to the node.
/// The node can then announce itself using that challenge.
/// TODO: use the "Challenge" to sign with the private key of the node, so that the server
/// can verify that the node knows the corresponding private key of its public key.
/// - ListIDs* are used by the nodes to get a list of currently connected nodes
/// - ClearNodes is a debugging message that will be removed at a later stage.
/// - PeerRequest is sent by a node to ask to connect to another node. The
/// server will send a 'PeerReply' to the corresponding node, which will continue
/// the protocol by sending its own PeerRequest.
/// - Done is a standard message that can be sent back to indicate all is well.
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Challenge(U256),
    Announce(NodeInfo),
    ListIDsRequest,
    ListIDsReply(Vec<NodeInfo>),
    ClearNodes,
    PeerRequest(PeerInfo),
    PeerReply(PeerInfo),
    Done,
}

pub struct WebRTCClient {
    rtc: Box<dyn WebRTCCaller>
}

impl WebRTCClient {
    pub fn new(rtc: Box<dyn WebRTCCaller>) -> WebRTCClient {
        WebRTCClient { rtc }
    }

    /// Returns the offer string that needs to be sent to the `Follower` node.
    pub async fn make_offer(&mut self) -> Result<String, String> {
        self.rtc
            .call(WebRTCMethod::MakeOffer, None)
            .await
            .map(|s| s.unwrap())
    }

    /// Takes the offer string
    pub async fn make_answer(&mut self, offer: String) -> Result<String, String> {
        self.rtc
            .call(WebRTCMethod::MakeAnswer, Some(offer))
            .await
            .map(|s| s.unwrap())
    }

    /// Takes the answer string and finalizes the first part of the connection.
    pub async fn use_answer(&mut self, answer: String) -> Result<(), String> {
        self.rtc
            .call(WebRTCMethod::UseAnswer, Some(answer))
            .await
            .map(|_| ())
    }

    /// Waits for the ICE to move on from the 'New' state
    pub async fn wait_gathering(&mut self) -> Result<(), String> {
        self.rtc
            .call(WebRTCMethod::WaitGathering, None)
            .await
            .map(|_| ())
    }

    /// Waits for the ICE string to be avaialble.
    pub async fn ice_string(&mut self) -> Result<String, String> {
        self.rtc
            .call(WebRTCMethod::IceString, None)
            .await
            .map(|s| s.unwrap())
    }

    /// Sends the ICE string to the WebRTC.
    pub async fn ice_put(&mut self, ice: String) -> Result<(), String> {
        self.rtc
            .call(WebRTCMethod::IcePut, Some(ice))
            .await
            .map(|_| ())
    }

    /// Waits for a message to arrive. If no message arrives within 10 * 100ms,
    /// an error is returned.
    pub async fn msg_receive(&mut self) -> Result<String, String> {
        self.rtc
            .call(WebRTCMethod::MsgReceive, None)
            .await
            .map(|s| s.unwrap())
    }

    /// Sends a message to the other end. If the DataChannel is not set up yet,
    /// it needs to wait for it to happen. If the setup doesn't happen within 10 * 1s,
    /// an error is returned.
    pub async fn msg_send(&mut self, s: String) -> Result<(), String> {
        self.rtc
            .call(WebRTCMethod::MsgSend, Some(s))
            .await
            .map(|_| ())
    }

    /// Debugging output of the RTC state
    pub async fn print_states(&mut self) {
        self.rtc.call(WebRTCMethod::MakeOffer, None).await.unwrap();
    }
}
