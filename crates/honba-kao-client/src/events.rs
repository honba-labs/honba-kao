//! SSE consumer for `GET /backtests/{id}/events?resume_from_seq=N` (ADR 0021).
//!
//! Transport decision (P0, HBD-2026-DES-005): the stream is opened through the
//! browser-native **`web_sys::EventSource`**. That is deliberate, not
//! incidental:
//!
//! - the server (honba-ya `streams.rs`) writes each `StreamFrame` as the `data:`
//!   payload with `id:` = the frame's `seq` and answers a header-free GET, so
//!   `EventSource` works through the Trunk dev proxy with no auth plumbing and
//!   reuses the browser's automatic reconnect (which re-issues the *original*
//!   URL — so an explicit `resume_from_seq`, refreshed by the consumer, is how
//!   no record is replayed or skipped);
//! - a fetch-stream reader would have to hand-parse `data:`/`id:` framing and
//!   keep-alive comments for no benefit.
//!
//! The frames are **bare `StreamFrame` JSON, not enveloped** — the SSE route
//! answers outside the `ResponseEnvelope` (the plain-text/event-stream
//! transport has no envelope; version pinning arrived when the journal records
//! themselves were written). Decode failures surface as `ClientError::Decode`
//! through the callback instead of aborting the stream.
//!
//! The ("the Leptos port of honba-fe …") note: this is the port of honba-fe's
//! `core/simulator/events.ts` SSE splice; the fold itself (`frame_to_line`,
//! `fold_run_log`) lives in honba-kao-app's `sim/log.rs` (P4).

use wasm_bindgen::JsCast;
use web_sys::{EventSource, MessageEvent};

use crate::{ClientError, wire::StreamFrame};

/// One open SSE connection to a run's event tail.
///
/// The `EventSource` and its `message` listener live as long as this handle;
/// call [`close`](RunEventStream::close) (or drop the handle) to stop the flow.
pub struct RunEventStream {
    source: EventSource,
    /// Keeps the closure alive; the `EventSource` holds only a raw reference.
    _on_message: wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>,
}

impl RunEventStream {
    /// Opens `GET {base_url}/backtests/{run_id}/events` and starts streaming
    /// decoded [`StreamFrame`]s into `on_frame` on the browser's event loop.
    ///
    /// `resume_from_seq` is the 0-based record index to replay from; the
    /// server acks with a `Status` frame whose `seq` is the next unread line,
    /// which consumers fold into their log state and reuse as the resume
    /// token after a reconnect.
    pub fn open(
        base_url: &str,
        run_id: &str,
        resume_from_seq: Option<u64>,
        mut on_frame: impl FnMut(Result<StreamFrame, ClientError>) + 'static,
    ) -> Result<Self, ClientError> {
        let mut url = format!("{}/backtests/{}/events", base_url, crate::encode_query(run_id));
        if let Some(seq) = resume_from_seq {
            url.push_str(&format!("?resume_from_seq={seq}"));
        }
        let source = EventSource::new(&url).map_err(|e| {
            ClientError::Network(format!("EventSource({url}): {e:?}"))
        })?;

        let on_message = {
            let callback = wasm_bindgen::closure::Closure::<dyn FnMut(MessageEvent)>::new(
                move |event: MessageEvent| {
                    let data = event.data();
                    let Some(text) = data.as_string() else {
                        on_frame(Err(ClientError::Decode(
                            "sse frame data is not a string".to_string(),
                        )));
                        return;
                    };
                    match serde_json::from_str::<StreamFrame>(&text) {
                        Ok(frame) => on_frame(Ok(frame)),
                        // Keep-alive comments and unknown frames are skipped,
                        // not fatal: the run log degrades, it never hangs.
                        Err(e) => on_frame(Err(ClientError::Decode(format!("frame: {e}")))),
                    }
                },
            );
            source
                .add_event_listener_with_callback("message", callback.as_ref().unchecked_ref())
                .map_err(|e| ClientError::Network(format!("listen: {e:?}")))?;
            callback
        };

        Ok(Self {
            source,
            _on_message: on_message,
        })
    }

    /// Closes the connection. Idempotent.
    pub fn close(&self) {
        self.source.close();
    }
}

impl Drop for RunEventStream {
    fn drop(&mut self) {
        self.source.close();
    }
}
