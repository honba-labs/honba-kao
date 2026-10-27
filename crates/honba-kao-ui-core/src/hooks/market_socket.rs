//! Browser WebSocket for the live market stream.
//!
//! `use_tick_stream` (in `hooks/mod.rs`) only reports messages. Panels that
//! need to know whether the feed is up use this hook, which also reports the
//! connection state, so the UI can fall back to labelled synthetic data when the
//! backend is unreachable.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Connection lifecycle reported to the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketState {
    Connecting,
    Open,
    /// The server closed the socket (after it was open or while connecting).
    Closed,
    /// The socket could not be created or raised an error.
    Failed,
}

/// Owns the socket and its JS callbacks. Dropped together with the owner.
struct SocketHandles {
    ws: web_sys::WebSocket,
    _open: Closure<dyn FnMut(web_sys::Event)>,
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(web_sys::Event)>,
    _close: Closure<dyn FnMut(web_sys::Event)>,
}

/// Opens `url` once for the lifetime of the calling owner. When the socket
/// opens and `subscribe` is `Some`, that text is sent as the first frame.
/// Every text frame goes to `on_text`; lifecycle changes go to `on_state`.
pub fn use_market_socket(
    url: String,
    subscribe: Option<String>,
    on_text: Callback<String>,
    on_state: Callback<SocketState>,
) {
    Effect::new(move |_| {
        on_state.run(SocketState::Connecting);
        let ws = match web_sys::WebSocket::new(&url) {
            Ok(ws) => ws,
            Err(_) => {
                on_state.run(SocketState::Failed);
                return;
            }
        };

        let ws_for_open = ws.clone();
        let subscribe = subscribe.clone();
        let open = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            on_state.run(SocketState::Open);
            if let Some(msg) = &subscribe {
                let _ = ws_for_open.send_with_str(msg);
            }
        });
        ws.set_onopen(Some(open.as_ref().unchecked_ref()));

        let message =
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                if let Some(text) = ev.data().as_string() {
                    on_text.run(text);
                }
            });
        ws.set_onmessage(Some(message.as_ref().unchecked_ref()));

        let error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            on_state.run(SocketState::Failed);
        });
        ws.set_onerror(Some(error.as_ref().unchecked_ref()));

        let close = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            on_state.run(SocketState::Closed);
        });
        ws.set_onclose(Some(close.as_ref().unchecked_ref()));

        let handles = StoredValue::new_local(SocketHandles {
            ws,
            _open: open,
            _message: message,
            _error: error,
            _close: close,
        });
        on_cleanup(move || {
            handles.with_value(|h| {
                h.ws.set_onopen(None);
                h.ws.set_onmessage(None);
                h.ws.set_onerror(None);
                h.ws.set_onclose(None);
                let _ = h.ws.close();
            });
            handles.dispose();
        });
    });
}
