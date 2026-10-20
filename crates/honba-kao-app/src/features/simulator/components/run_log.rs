//! The run log & events drawer — the Leptos port of honba-fe
//! `features/simulator/components/run-log-drawer.tsx` (port phase P5, ADR
//! 0021).
//!
//! The poll loop already leaves a progress log (`run.logs`); on top of it
//! this drawer tails `GET /backtests/{id}/events` (SSE, opened through the
//! P0 [`RunEventStream`]) whenever a run id exists. Folding happens through
//! `sim::log`, and the fold is deliberately **ordered by seq**: the
//! browser's automatic reconnect re-issues the *original*
//! `resume_from_seq` URL, so replayed frames are already folded and are
//! skipped by sequence instead of duplicated. The drawer never blocks the
//! bench when the route is missing — it just shows the poll log.
//!
//! Parking note: a parked `EventSource` handle is neither `Send` nor
//! `Sync` (so it cannot sit inside a signal); it lives in a thread-local
//! registry keyed by id, and the *signal* stores that id. Closing is
//! dropping the parked handle — the client crate's wrapper closes the
//! `EventSource` on drop.

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::prelude::*;
use crate::features::simulator::kernel::log::{initial_run_log, RunLogState};
use crate::features::simulator::kernel::run::{SimRunState, SimRunStatus};

/// The client's SSE consumer: the fold lives here, the transport in the
/// client crate (`RunEventStream`). Only reachable under wasm — the native
/// build has no `EventSource` runtime, so the drawer's poll log stands
/// alone there exactly like fe's no-route fallback.
#[cfg(target_arch = "wasm32")]
use crate::shared::api::http::base_url;
#[cfg(target_arch = "wasm32")]
use crate::features::simulator::kernel::log::fold_run_log;
#[cfg(target_arch = "wasm32")]
use honba_kao_client::events::RunEventStream;
#[cfg(target_arch = "wasm32")]
use honba_kao_client::wire::StreamFrame;

/// One open stream handle (only exists under wasm; the native alias holds
/// nothing so the component still compiles and never lies about a stream).
#[cfg(target_arch = "wasm32")]
type StreamHandle = Rc<RunEventStream>;
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
type StreamHandle = ();

// Parked streams keyed by ticket id: entries are removed (dropping the
// handle and closing the EventSource) on every reactive reset and on
// on_cleanup; the drawer parks at most one stream at a time.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static PARKED: RefCell<HashMap<usize, StreamHandle>> = RefCell::new(HashMap::new());
}
#[cfg(target_arch = "wasm32")]
static STREAM_TICKETS: AtomicUsize = AtomicUsize::new(1);

/// Parks one stream and returns its ticket id (`None`-free wasm path).
#[cfg(target_arch = "wasm32")]
fn park_stream(handle: StreamHandle) -> usize {
    let ticket = STREAM_TICKETS.fetch_add(1, Ordering::Relaxed);
    PARKED.with(|parked| parked.borrow_mut().insert(ticket, handle));
    ticket
}

/// Un-parks a ticket, dropping the handle and closing its `EventSource`.
fn unpark_stream(_ticket: Option<usize>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(ticket) = _ticket {
            PARKED.with(|parked| parked.borrow_mut().remove(&ticket));
        }
    }
    // Native: nothing was ever parked.
    let _ = _ticket;
}

/// Opens one SSE tail for `run_id`, folding frames into `live` under the
/// seq guard, parked in the thread-local registry. `None` when the platform
/// has no stream (native) or the route refused the connection — the drawer
/// then shows the poll log only and never blocks the bench.
fn open_stream(run_id: &str, live: RwSignal<RunLogState>) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        let handle: StreamHandle = Rc::new(RunEventStream::open(&base_url(), run_id, None, move |frame| {
            let Ok(frame) = frame else {
                // Decode failures are swallowed like fe's fold: the poll log
                // still shows, the tail retries silently.
                return;
            };
            live.update(|state| {
                // Replayed frames are already folded — skip them by seq so
                // the browser's auto-reconnect (which re-issues the original
                // `resume_from_seq` URL) never duplicates lines.
                let seq = match &frame {
                    StreamFrame::Status { seq, .. } | StreamFrame::Record { seq, .. } => *seq,
                };
                if seq <= state.last_seq {
                    return;
                }
                *state = fold_run_log(state, Some(&frame));
            });
        })
        .ok()?);
        Some(park_stream(handle))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (run_id, live);
        None
    }
}

/// The collapse-state run log + event tail (`RunLogDrawer`).
#[component]
pub fn RunLogDrawer(
    /// The run lifecycle signal — statuses, run id and poll lines (`run`).
    run: RwSignal<SimRunState>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let live = RwSignal::new(initial_run_log());
    let streaming = RwSignal::new(false);
    // The ticket id (not the handle) travels in the signal: the parked
    // handle is neither Send nor Sync.
    let stream_ticket = RwSignal::new(None::<usize>);

    // fe's effect deps were [open, runId, client, status]: the tail rebuilds
    // on open, on run change and on each status transition.
    let run_line = Memo::new(move |_| run.with(|state| state.run_id.clone().unwrap_or_default()));
    let status_line = Memo::new(move |_| run.with(|state| state.status.as_str().to_string()));

    Effect::new(move |_| {
        let open = open.get();
        let run_id = run_line.get();
        let _status = status_line.get();

        // A fresh tail per (open × run): fe reset the live log here, so the
        // stream below is opened from the runid's event store start; the seq
        // guard keeps a *reconnect over the same stream* from duplicating,
        // exactly as the transport's design note requires.
        live.set(initial_run_log());
        streaming.set(false);
        unpark_stream(stream_ticket.get_untracked());
        stream_ticket.set(None);

        if !open || run_id.is_empty() || run.get_untracked().status == SimRunStatus::Idle {
            return;
        }

        if let Some(ticket) = open_stream(&run_id, live) {
            streaming.set(true);
            stream_ticket.set(Some(ticket));
        } else {
            // No stream (native, or the route refused): the poll log stands
            // alone (the drawer never blocks on the stream).
            streaming.set(false);
        }
    });

    on_cleanup(move || unpark_stream(stream_ticket.get_untracked()));

    let poll_count = Memo::new(move |_| run.with(|state| state.logs.len()));
    let line_count =
        Memo::new(move |_| poll_count.get() + live.with(|state| state.lines.len()));

    let poll_lines = Memo::new(move |_| run.with(|state| state.logs.clone()));
    let live_lines = Memo::new(move |_| live.with(|state| state.lines.clone()));

    let badge = move || {
        if streaming.get() {
            ("streaming…".to_string(), "sim-log__badge sim-log__badge--live")
        } else if run_line.get().is_empty() {
            ("poll only".to_string(), "sim-log__badge sim-log__badge--muted")
        } else {
            ("SSE".to_string(), "sim-log__badge sim-log__badge--muted")
        }
    };

    view! {
        <div class="sim-log" data-testid="run-log-drawer">
            <button
                type="button"
                class="sim-log__toggle"
                aria-expanded=move || open.get()
                on:click=move |_| open.update(|open| *open = !*open)
            >
                <span class="sim-log__toggle-name">
                    {move || if open.get() { "▾" } else { "▸" }}
                    "Run log & events"
                    <span class="sim-log__count">
                        {move || format!("({})", line_count.get())}
                    </span>
                </span>
                {move || {
                    let (text, class) = badge();
                    view! { <span class=class>{text}</span> }
                }}
            </button>

            {move || {
                if !open.get() {
                    view! { <span /> }.into_any()
                } else {
                    let lines = [poll_lines.get(), live_lines.get()].concat();
                    view! {
                        <div class="sim-log__lines">
                            {if lines.is_empty() {
                                view! { <span class="sim-log__empty">"No run activity yet."</span> }
                                    .into_any()
                            } else {
                                lines
                                    .into_iter()
                                    .map(|line| view! { <span class="sim-log__line">{line}</span> })
                                    .collect_view()
                                    .into_any()
                            }}
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
