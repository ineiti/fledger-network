use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use common::{ext_interface::Logger, websocket::WebSocketConnectionSend};
use common::types::U256;
use common::web_rtc::Message;
use common::websocket::WebSocketServer;

mod internal;
mod node_entry;
use internal::Internal;
use node_entry::NodeEntry;

pub struct ServerState {
    int: Arc<Mutex<Internal>>,
}

/// This holds the logic of the signalling server.
/// It can do the following;
/// - listen for incoming websocket requests
/// - handle webrtc signalling setup
impl ServerState {
    pub fn new(logger: Box<dyn Logger>, mut ws: Box<dyn WebSocketServer>) -> ServerState {
        let int = Internal::new(logger);
        let int_cl = Arc::clone(&int);
        ws.set_cb_connection(Box::new(move |conn| {
            ServerState::cb_connection(Arc::clone(&int_cl), conn)
        }));
        ServerState { int }
    }

    /// Treats new connections from websockets.
    fn cb_connection(int: Arc<Mutex<Internal>>, mut conn: Box<dyn WebSocketConnectionSend + Send>) {
        let challenge = U256::rnd();
        let ch_cl = challenge.clone();
        let int_clone = Arc::clone(&int);
        conn.set_cb_wsmessage(Box::new(move |cb| {
            int_clone.lock().unwrap().cb_msg(&ch_cl, cb)
        }));

        let mut int_lock = int.lock().unwrap();
        let logger = int_lock.logger.clone();
        int_lock
            .nodes
            .insert(challenge.clone(), NodeEntry::new(logger, challenge, conn));
    }

    /// Bogous wait for all done.
    pub fn wait_done(&self) {
        loop {
            thread::park();
        }
    }

    // pub fn send_message(&self, entry: &U256, msg: Message) {
    //     let int_arc = Arc::clone(&self.int);
    //     let mut int = int_arc.lock().unwrap();
    //     int.send_message(entry, msg);
    // }
}

#[cfg(test)]
mod tests {}
