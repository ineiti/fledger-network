use common::ext_interface::{DataStorage, WebRTCConnection};
use common::ext_interface::Logger;

use common::ext_interface::WebRTCConnectionState;

use common::network::Network;use common::node::Node;
use wasm_bindgen::JsValue;
use web_sys::window;

use crate::web_rtc::WebRTCConnectionWasm;use crate::web_socket::WebSocketWasm;


struct MyDataStorage {}

impl DataStorage for MyDataStorage {
    fn load(&self, key: &str) -> Result<String, String> {
        window()
            .unwrap()
            .local_storage()
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .get(key)
            .map(|s| s.unwrap_or("".to_string()))
            .map_err(|e| e.as_string().unwrap())
    }

    fn save(&self, key: &str, value: &str) -> Result<(), String> {
        window()
            .unwrap()
            .local_storage()
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .set(key, value)
            .map_err(|e| e.as_string().unwrap())
    }
}

struct WasmLogger{}

impl Logger for WasmLogger{
    fn info(&self, s: &str) {
        console_log!("info: {}", s);
    }

    fn warn(&self, s: &str) {
        console_warn!("warn: {}", s);
    }

    fn error(&self, s: &str) {
        console_warn!(" err: {}", s);
    }

    fn clone(&self) -> Box<dyn Logger> {
        Box::new(WasmLogger{})
    }
}

fn webrtc_spawner(cs: WebRTCConnectionState) -> Result<Box<dyn WebRTCConnection>, String>{
    WebRTCConnectionWasm::new(cs).map(|conn| Box::new(conn))
}

pub async fn start(log: Box<dyn Logger>, url: &str) -> Result<Node, JsValue> {
    let rtc_spawner = WebRTCConnectionWasm::new(WebRTCConnectionState::Initializer)?;
    let my_storage = Box::new(MyDataStorage {});
    // let ws = WebSocketWasm::new("wss://signal.fledg.re")?;
    let ws = WebSocketWasm::new(url)?;
    let logger = WasmLogger{};
    let network = Network::new(Box::new(ws), Box::new(rtc_spawner), Box::new(logger));
    let node = Node::new(my_storage, log, network)?;

    Ok(node)
}
