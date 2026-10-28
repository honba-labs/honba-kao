//! Server-Sent Events stream for AI thought steps and tool calls.
//!
//! Delivers messages through a channel (`SseStream`) so consumers can bind
//! them to Leptos reactive signals rather than raw closures.

use crate::stream::SseStream;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentStep {
    pub agent_name: String,
    pub step_type: String,
    pub content: String,
    pub timestamp: String,
}

pub fn connect_agent_stream(url: &str) -> Result<SseStream<AgentStep>, String> {
    let es =
        EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let (tx, rx) = futures_channel::mpsc::unbounded::<AgentStep>();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(step) = serde_json::from_str::<AgentStep>(&txt) {
                let _ = tx.unbounded_send(step);
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
