use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

pub struct WsConnection {
    pub ws: WebSocket,
}

impl WsConnection {
    pub fn connect<F>(url: &str, mut on_message: F) -> Result<Self, String>
    where
        F: FnMut(String) + 'static,
    {
        let ws = WebSocket::new(url).map_err(|e| format!("WebSocket creation failed: {e:?}"))?;

        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Some(txt) = e.data().as_string() {
                on_message(txt);
            }
        });

        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        Ok(Self { ws })
    }

    pub fn send(&self, text: &str) -> Result<(), String> {
        self.ws
            .send_with_str(text)
            .map_err(|e| format!("WebSocket send error: {e:?}"))
    }

    pub fn close(&self) -> Result<(), String> {
        self.ws
            .close()
            .map_err(|e| format!("WebSocket close error: {e:?}"))
    }
}
