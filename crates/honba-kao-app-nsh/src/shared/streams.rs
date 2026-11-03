//! Reactive NSH bindings for headless SSE/WebSocket streams, split by
//! platform.
//!
//! `honba-kao-client` is intentionally headless: it owns the transports
//! and exposes messages through channels (`SseStream`). This module
//! bridges those channels into NSH reactive signals so the application
//! shell can drive the UI from live feeds.
//!
//! Stage 3 of the gallery-rs port (Phase A.1 "decouple platform layer
//! from web-sys"): every browser sink lives behind
//! `#[cfg(target_arch = "wasm32")]`. The pool's [`StreamHandle`] is the
//! seam between the two runtimes:
//!
//! - **Web runtime** (`wasm32`): the handle is the live
//!   `web_sys::EventSource` — it must be retained for the connection to
//!   stay open (dropping it closes the SSE).
//! - **Native desktop runtime**: there is no browser `EventSource`, so
//!   the handle is an inert marker and no transport is started. The app
//!   shells run disconnected (offline mode) until a native transport
//!   lands; `HonbaClientContext` renders its disconnected states.
//!
//! NSH port (nsh-port.md Phase 5): `wasm_bindgen_futures::spawn_local`
//! became `nsh_core::prelude::spawn` (runs the future under the current
//! `Owner` scope rather than a bare wasm microtask queue);
//! `leptos::prelude::RwSignal` became `nsh_core::prelude::Signal`, which
//! has the same `Copy` + `.update(|v| ..)` mutate-in-place semantics, so
//! the push-onto-`Vec` binding below is unchanged apart from the type
//! name.

/// The retained SSE connection handle (see the module docs).
#[cfg(target_arch = "wasm32")]
pub type StreamHandle = web_sys::EventSource;

/// Native twin of [`StreamHandle`]: an inert marker. The retaining
/// struct-field stays valid across both runtimes (mobile `Option<_>` for
/// the web handle) while the desktop runtime keeps no live transport.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StreamHandle;

/// Bind an SSE stream to an NSH signal, appending every received message
/// to a `Vec`.
///
/// Web runtime only: the stream is consumed (its receiver is moved into
/// a spawned task), so call this once per binding. Returns the underlying
/// [`StreamHandle`] (the `EventSource`) so the caller can close it when
/// the owning component is torn down.
#[cfg(target_arch = "wasm32")]
pub fn bind_sse_stream<T: PartialEq + Clone + 'static>(
    stream: honba_kao_client::stream::SseStream<T>,
    signal: nsh_core::prelude::Signal<Vec<T>>,
) -> StreamHandle {
    use futures_util::StreamExt;
    use nsh_core::prelude::spawn;

    let (event_source, mut receiver) = stream.into_parts();

    spawn(async move {
        while let Some(msg) = receiver.next().await {
            signal.update(|msgs| msgs.push(msg));
        }
    });

    event_source
}
