use common::web_rtc::{WebRTCConnection, WebRTCMessageCB};
use wasm_bindgen::{ JsValue ,JsCast, prelude::Closure};
use web_sys::{console::log_1, MessageEvent, RtcDataChannel};

fn log(s: &str) {
    log_1(&JsValue::from_str(s));
}

pub struct WebRTCConnectionWasm {
    dc: RtcDataChannel,
}

impl WebRTCConnectionWasm {
    pub fn new(dc: RtcDataChannel) -> Box<dyn WebRTCConnection> {
        Box::new(WebRTCConnectionWasm { dc })
    }
}

impl WebRTCConnection for WebRTCConnectionWasm {
    /// Send a message to the other node. This call blocks until the message
    /// is queued.
    fn send(&self, s: String) -> Result<(), String> {
        self.dc.send_with_str(&s).map_err(|e| e.as_string().unwrap())
    }

    /// Sets the callback for incoming messages.
    fn set_cb_message(&self, mut cb: WebRTCMessageCB) {
        let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
            log(&format!("New event: {:?}", ev));
            match ev.data().as_string() {
                Some(message) => {
                    log(&format!("got: {:?}", message));
                    cb(message);
                }
                None => {}
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        self.dc
            .set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }
}
