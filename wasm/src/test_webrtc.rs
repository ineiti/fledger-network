use common::ext_interface::Logger;
use common::node::Node;

use common::types::U256;
use common::web_rtc::WSSignalMessage;
use common::web_rtc::WebSocketMessage;
use common::websocket::MessageCallback;
use common::websocket::WSMessage;
use std::cell::RefCell;
use std::rc::Rc;

use common::websocket::WebSocketConnection;
use wasm_bindgen_test::*;

use crate::node::MyDataStorage;

use crate::node::WasmLogger;
use crate::web_rtc_setup::WebRTCConnectionSetupWasm;

struct Message {
    str: String,
    id: u32,
}

struct WebSocketDummy {
    msg_queue: Rc<RefCell<Vec<Message>>>,
    connections: Vec<WebSocketConnectionDummy>,
    logger: Box<dyn Logger>,
}

impl WebSocketDummy {
    fn new(logger: Box<dyn Logger>) -> WebSocketDummy {
        WebSocketDummy {
            msg_queue: Rc::new(RefCell::new(vec![])),
            connections: vec![],
            logger,
        }
    }

    fn get_connection(&mut self) -> Result<WebSocketConnectionDummy, String> {
        let id = (self.connections.len() + 1) as u32;
        if id > 2 {
            return Err("currently only supports 2 nodes".to_string());
        }
        let wscd = WebSocketConnectionDummy::new(Rc::clone(&self.msg_queue), id);
        let wscd_clone = wscd.clone();
        self.connections.push(wscd);
        let wsm_str = WebSocketMessage {
            msg: WSSignalMessage::Challenge(U256::rnd()),
        }
        .to_string();
        self.msg_queue
            .borrow_mut()
            .push(Message { id, str: wsm_str });
        Ok(wscd_clone)
    }

    fn run_queue(&mut self) -> Result<(), String> {
        let msgs: Vec<Message> = self.msg_queue.borrow_mut().drain(..).collect();
        msgs.iter().for_each(|msg| {
            if let Ok(wsm) = WebSocketMessage::from_str(&msg.str) {
                self.logger
                    .info(&format!("Got msg {:?} from {}", wsm.msg, msg.id));
                match wsm.msg {
                    WSSignalMessage::PeerSetup(_) => {
                        if self.connections.len() == 2 {
                            if let Err(e) = self.connections
                                .get_mut(2 - msg.id as usize)
                                .unwrap()
                                .push_message(msg.str.clone()){
                                    self.logger.error(&format!("couldn't push message: {}", e));
                                }
                        }
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }
}

struct WebSocketConnectionDummy {
    msg_queue: Rc<RefCell<Vec<Message>>>,
    id: u32,
    cb: Option<MessageCallback>,
}

impl WebSocketConnectionDummy {
    fn new(msg_queue: Rc<RefCell<Vec<Message>>>, id: u32) -> WebSocketConnectionDummy {
        WebSocketConnectionDummy {
            msg_queue,
            id,
            cb: None,
        }
    }

    fn push_message(&mut self, msg: String) -> Result<(), String> {
        if self.cb.is_none() {
            return Err("no callback defined yet".to_string());
        }
        (self.cb.as_mut().unwrap())(WSMessage::MessageString(msg));
        Ok(())
    }

    fn clone(&self) -> WebSocketConnectionDummy{
        WebSocketConnectionDummy{
            msg_queue: Rc::clone(&self.msg_queue),
            id: self.id,
            cb: None,
        }
    }
}

impl WebSocketConnection for WebSocketConnectionDummy {
    fn set_cb_wsmessage(&mut self, cb: MessageCallback) {
        self.cb.replace(cb);
    }

    fn send(&mut self, msg: String) -> Result<(), String> {
        let queue = Rc::clone(&self.msg_queue);
        queue.borrow_mut().push(Message {
            id: self.id,
            str: msg.clone(),
        });
        Ok(())
    }
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn connect_test() {
    let log = Box::new(WasmLogger {});
    match connect_test_simple().await {
        Ok(_) => log.info("All OK"),
        Err(e) => log.error(&format!("Something went wrong: {}", e)),
    };
}

async fn connect_test_simple() -> Result<(), String> {
    let log = Box::new(WasmLogger {});
    let mut ws_conn = WebSocketDummy::new(log.clone());

    // First node
    let rtc_spawner = Box::new(|cs| WebRTCConnectionSetupWasm::new(cs));
    let my_storage = Box::new(MyDataStorage {});
    let ws = Box::new(ws_conn.get_connection()?);
    let node1 = Node::new(my_storage, log.clone(), ws, rtc_spawner)?;

    // Second node
    let rtc_spawner = Box::new(|cs| WebRTCConnectionSetupWasm::new(cs));
    let my_storage = Box::new(MyDataStorage {});
    let ws = Box::new(ws_conn.get_connection()?);
    let node2 = Node::new(my_storage, log.clone(), ws, rtc_spawner)?;

    // Pass messages
    ws_conn.run_queue()?;
    let p = "ping".to_string();
    node1.send(&node2.info.public, p).await?;
    Ok(())
}
