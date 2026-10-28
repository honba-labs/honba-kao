//! Server-Sent Events (SSE) connections for live user order state, margin
//! breach warnings, and AI thought steps.
//!
//! Streams deliver messages through a channel (`SseStream`) so consumers can
//! bind them to Leptos reactive signals rather than raw closures.

use futures_channel::mpsc::UnboundedReceiver;
use wasm_bindgen::prelude::Closure;
use web_sys::MessageEvent;

pub mod agent_stream;
pub mod orders_stream;
pub mod risk_stream;

pub use agent_stream::{AgentStep, connect_agent_stream};
pub use orders_stream::connect_orders_stream;
pub use risk_stream::connect_risk_stream;

/// A headless SSE stream: the underlying `EventSource` plus a channel receiver
/// that yields deserialized messages as they arrive.
///
/// #67: the registered message `Closure` is retained for the stream's lifetime
/// and `Drop` closes the `EventSource`, so unmounting a Leptos component no
/// longer leaves a zombie HTTP connection and a leaked callback firing into a
/// disconnected channel.
pub struct SseStream<T> {
    pub event_source: web_sys::EventSource,
    pub receiver: UnboundedReceiver<T>,
    _closure: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl<T> SseStream<T> {
    /// Consume the stream, returning its parts. The caller takes ownership of
    /// the `EventSource` (responsible for closing it, e.g. in a Leptos
    /// `on_cleanup`); the message `Closure` is released here.
    ///
    /// Use this when the caller manages the `EventSource` lifecycle itself.
    /// When an `SseStream` is simply dropped (not consumed), [`Drop`] closes
    /// the `EventSource` and releases the callback (#67), preventing zombie
    /// connections.
    pub fn into_parts(self) -> (web_sys::EventSource, UnboundedReceiver<T>) {
        let this = std::mem::ManuallyDrop::new(self);
        // Bitwise-relocate the handle fields out and forget `this` so `Drop`
        // does not run a second time (it would close the `EventSource` we are
        // handing over). `EventSource` and `UnboundedReceiver` are
        // JS-object/channel handles; bitwise relocation is sound because no
        // resource-releasing `Drop` runs on the source copies once forgotten.
        let event_source = unsafe { std::ptr::read(&this.event_source) };
        let receiver = unsafe { std::ptr::read(&this.receiver) };
        let closure = unsafe { std::ptr::read(&this._closure) };
        std::mem::forget(this);
        drop(closure); // release the JS callback
        (event_source, receiver)
    }
}

impl<T> Drop for SseStream<T> {
    fn drop(&mut self) {
        // Stop the underlying HTTP connection first.
        self.event_source.close();
        // Release the JS callback so it can no longer fire into the (now
        // disconnected) channel receiver.
        self._closure.take();
    }
}
