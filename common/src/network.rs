use crate::ext_interface::Logger;
use crate::ext_interface::WebRTCCaller;

use crate::types::U256;
use crate::websocket::WSMessage;
use crate::websocket::WebSocketConnection;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Network {
    intern: Arc<Mutex<Intern>>,
}

struct Intern {
    ws: Box<dyn WebSocketConnection>,
    web_rtc: Box<dyn WebRTCCaller>,
    logger: Box<dyn Logger>,
}

impl Intern {
    pub fn msg_cb(&mut self, msg: WSMessage) {
        match msg{
            WSMessage::MessageString(s) => {
                self.logger.info(&format!("Got a message: {:?}", s));
            }
            WSMessage::Closed(_) => {}
            WSMessage::Opened(_) => {}
        }
    }
}

/// Network combines a websocket to connect to the signal server with
/// a WebRTC trait to connect to other nodes.
/// It supports setting up automatic connetions to other nodes.
impl Network {
    pub fn new(
        ws: Box<dyn WebSocketConnection>,
        web_rtc: Box<dyn WebRTCCaller>,
        logger: Box<dyn Logger>,
    ) -> Network {
        let net = Network {
            intern: Arc::new(Mutex::new(Intern {
                ws,
                web_rtc,
                logger,
            })),
        };
        let n = Arc::clone(&net.intern);
        net.intern
            .lock()
            .unwrap()
            .ws
            .set_cb_wsmessage(Box::new(move |msg| n.lock().unwrap().msg_cb(msg)));
        net
    }

    /// Sending strings to other nodes. If the connection already exists,
    /// it will  be used to send the string over.
    /// Else the signalling server will be contacted, a webrtc connection will
    /// be created, and then the message will be sent over.
    pub async fn send(&self, dst: U256, msg: String) -> Result<(), String> {
        let _ = Arc::clone(&self.intern).lock().unwrap().ws.send(msg);
        Ok(())
    }
}
