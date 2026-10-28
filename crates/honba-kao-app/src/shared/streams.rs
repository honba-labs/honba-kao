//! Reactive Leptos bindings for headless SSE/WebSocket streams.
//!
//! `honba-kao-client` is intentionally headless: it owns the transports and
//! exposes messages through channels (`SseStream`). This module bridges those
//! channels into Leptos reactive signals so the application shell can drive
//! the UI from live feeds.

use futures_util::StreamExt;
use honba_kao_client::stream::SseStream;
use leptos::RwSignal;
use wasm_bindgen_futures::spawn_local;

/// Bind an SSE stream to a Leptos signal, appending every received message to
/// a `Vec`.
///
/// The stream is consumed (its receiver is moved into a local task), so call
/// this once per binding. Returns the underlying `EventSource` so the caller
/// can close it (e.g. in `on_cleanup`) when the component unmounts.
pub fn bind_sse_stream<T: 'static>(
    stream: SseStream<T>,
    signal: RwSignal<Vec<T>>,
) -> web_sys::EventSource {
    let event_source = stream.event_source;
    let mut receiver = stream.receiver;

    spawn_local(async move {
        while let Some(msg) = receiver.next().await {
            signal.update(|msgs| msgs.push(msg));
        }
    });

    event_source
}
