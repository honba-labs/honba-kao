//! Server-Sent Events stream for real-time margin breach warnings.
//!
//! Delivers messages through a channel (`SseStream`) so consumers can bind
//! them to Leptos reactive signals rather than raw closures.

use crate::dto::risk::LiquidationWarning;
use crate::stream::SseStream;
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

pub fn connect_risk_stream(url: &str) -> Result<SseStream<LiquidationWarning>, String> {
    let es =
        EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let (tx, rx) = futures_channel::mpsc::unbounded::<LiquidationWarning>();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(warning) = serde_json::from_str::<LiquidationWarning>(&txt) {
                let _ = tx.unbounded_send(warning);
            }
        }
    });

    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    Ok(SseStream {
        event_source: es,
        receiver: rx,
        _closure: Some(onmessage),
    })
}
