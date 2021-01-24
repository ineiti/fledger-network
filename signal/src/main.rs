/// Very simple rendez-vous server that allows new nodes to send their node_info.
/// It also allows the nodes to fetch all existing node_infos of all the other nodes.
///
/// TODO: use the `newID` endpoint to authentify the nodes' public key
// mod node_list;
mod state;

use std::net::TcpListener;

use common::ext_interface::Logger;

use async_trait::async_trait;

use common::websocket::MessageCallback;
use common::websocket::WebSocketConnection;use state::ServerState;

pub struct DummyWebSocket {}

impl DummyWebSocket {
    fn new() -> DummyWebSocket {
        let server = TcpListener::bind("127.0.0.1:8080").unwrap();
        for stream in server.incoming() {
            // state.new_connection(stream);
        }
        DummyWebSocket {}
    }

    fn register_cb(&self, cb: MessageCallback) {}
}

pub struct DummyLogger {}

impl Logger for DummyLogger {
    fn info(&self, s: &str) {
        println!("{}", s);
    }

    fn warn(&self, s: &str) {
        println!("{}", s);
    }

    fn error(&self, s: &str) {
        println!("{}", s);
    }
}

fn main() {
    // let state = ServerState::new();
    let logger = Box::new(DummyLogger {});
    let ws = Box::new(DummyWebSocket::new());
    let state = ServerState::new(logger, ws);
    state.wait_done();
}

struct DummyWSConnection {}

#[async_trait(?Send)]
impl WebSocketConnection for DummyWSConnection {
    fn set_cb_wsmessage(&mut self, cb: MessageCallback) {
        todo!()
    }

    async fn send(&self, msg: String) -> Result<(), String> {
        todo!()
    }
}

// pub struct DummyWebRTC {}
//
// #[async_trait(?Send)]
// impl WebRTCCaller for DummyWebRTC {
//     async fn call(
//         &mut self,
//         call: WebRTCMethod,
//         input: Option<String>,
//     ) -> Result<Option<String>, String> {
//         Err("not implemented".to_string())
//     }
// }
