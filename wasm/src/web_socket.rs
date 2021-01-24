use async_trait::async_trait;


use common::web_rtc::WebSocketMessage;use common::websocket::MessageCallback;
use common::websocket::WebSocketConnection;use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use web_sys::ErrorEvent;
use web_sys::MessageEvent;
use web_sys::WebSocket;

pub struct WebSocketWasm {
    cb: Option<MessageCallback>,
    ws: WebSocket,
}

impl WebSocketWasm {
    pub fn new(addr: &str) -> Result<WebSocketWasm, JsValue> {
        // Connect to an echo server
        let ws = WebSocket::new(addr)?;

        // create callback
        let cloned_ws = ws.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                console_log!("message event, received Text: {:?}", txt);
            } else {
                console_log!("message event, received Unknown: {:?}", e);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            console_log!("error event: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let cloned_ws = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            console_log!("socket opened");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        Ok(WebSocketWasm { cb: None, ws })
    }
}

#[async_trait(?Send)]
impl WebSocketConnection for WebSocketWasm {
    async fn send(&self, msg: String) -> Result<(), String> {
        let _ = self.ws.send_with_str(&serde_json::to_string(&msg).map_err(|e| e.to_string())?);
        Ok(())
    }

    fn set_cb_wsmessage(&mut self, cb: MessageCallback) {
        todo!()
    }
}
