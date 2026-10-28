//! Polling tick — the Leptos equivalent of SWR's `refreshInterval`.
//!
//! Reading the returned signal inside a reactive closure re-runs it on every
//! tick, which is how screens refresh positions/account/decisions without a
//! client-side data-fetching library.

use core::time::Duration;
use leptos::prelude::*;

/// Returns a signal that increments every `period_ms`. The interval is cleared
/// when the owning scope is disposed.
pub fn use_interval(period_ms: u32) -> ReadSignal<u64> {
    let tick = RwSignal::new(0u64);

    if !cfg!(test)
        && let Ok(handle) = set_interval_with_handle(
            move || tick.update(|t| *t += 1),
            Duration::from_millis(period_ms as u64),
        )
    {
        on_cleanup(move || handle.clear());
    }

    tick.read_only()
}
