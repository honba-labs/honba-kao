//! The simulator's pure kernel (HBD-2026-DES-005 P1+).
//!
//! Pure Rust, no DOM, no leptos, no network: these modules are the direct
//! ports of honba-fe's `core/simulator/*` pure layers, each keeping its
//! vitest suite as in-file `#[cfg(test)]` tests. Presentation (`stores/`,
//! `components/simulator/`) consumes them; the reactive store (`stores/`) is
//! the only place that holds signals.
//!
//! Port map:
//! - `defaults.rs` — slider/select constants + fill-model descriptors
//!   (fe `controls.ts` constant blocks);
//! - `controls.rs` — control model → `BacktestRequest` planning
//!   (fe `controls.ts`);
//! - `chart_data.rs` (P2) — bars → chart series, resampling, preview
//!   (fe `chart-data.ts`, `bars.ts`);
//! - `persistence.rs` (P2) — `localStorage` handoff (fe `persistence.ts`);
//! - `pairing.rs` (P3) — fills → round trips (fe `trade-pairing.ts`);
//! - `analytics.rs` (P3) — honest metrics + heatmap + CSV (fe `analytics.ts`);
//! - `ops.rs` (P3/P5 helpers) — run-mode/trust badges + drawdown stats
//!   (fe `ops.ts`);
//! - `selection.rs` (P4) — strategy graph shape, `controls_for_graph`,
//!   `universe_string` (fe `select.ts`, minus the studio port's parts);
//! - `run.rs` (P4) — run state fold + submit/poll loop (fe `run.ts`);
//! - `journal.rs` (P4) — journal fetch fold (fe `journal.ts`);
//! - `log.rs` (P4) — run-event fold (fe `events.ts`);
//! - `manifest.rs` (P4) — graph → compile manifest + compile seam
//!   (fe algodesigner `manifest.ts`/`compile.ts`, the run-facing slice).

pub mod analytics;
pub mod bars;
pub mod chart_data;
pub mod controls;
pub mod defaults;
pub mod journal;
pub mod log;
pub mod manifest;
pub mod ops;
pub mod pairing;
pub mod persistence;
pub mod run;
pub mod selection;

/// Test-only executor for the injected client/clock futures: a `block_on`
/// with **no wakeup scheduling** — it spins `poll` until an immediately-ready
/// future resolves. That covers every test double in `sim/` (their futures
/// resolve on the first poll) and nothing else; never use it in app code.
#[cfg(test)]
pub(crate) mod testing {
    pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
        use std::pin::Pin;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        fn noop_waker() -> Waker {
            fn clone(_: *const ()) -> RawWaker {
                RawWaker::new(std::ptr::null(), &VTABLE)
            }
            fn drop(_: *const ()) {}
            const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, drop, drop, drop);
            unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
        }

        let mut future = Box::pin(future);
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        loop {
            match Pin::new(&mut future).as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::hint::spin_loop(),
            }
        }
    }
}

pub use controls::{
    control_issues, default_controls, minus_months, parse_bar_spec, plan_request,
    preset_range, resolve_range, universe_for, PeriodPreset, SimulatorControls, Today,
};
pub use defaults::{format_fill_model, format_timeframe, DEFAULT_BENCHMARK, DEFAULT_CAPITAL, DEFAULT_SEED};
