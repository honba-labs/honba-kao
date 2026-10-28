//! Server-Sent Events stream for live user order state changes and execution
//! reports.
//!
//! Delivers messages through a channel (`SseStream`) so consumers can bind
//! them to Leptos reactive signals rather than raw closures.

use crate::dto::order::OrderResponse;
use crate::stream::SseStream;
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

pub fn connect_orders_stream(url: &str) -> Result<SseStream<OrderResponse>, String> {
    let es =
        EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let (tx, rx) = futures_channel::mpsc::unbounded::<OrderResponse>();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(order) = serde_json::from_str::<OrderResponse>(&txt) {
                let _ = tx.unbounded_send(order);
            }
        }
    });

    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(SseStream {
        event_source: es,
        receiver: rx,
    })
}
