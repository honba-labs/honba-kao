//! The simulator's pure kernel (HBD-2026-DES-005 P1+).
//!
//! Pure Rust, no DOM, no leptos, no network: these modules are the direct
//! ports of honba-fe's `core/simulator/*` pure layers, each keeping its
//! vitest suite as in-file `#[cfg(test)]` tests. Presentation (`stores/`,
//! `components/`) consumes them; the reactive store (`stores/`) is
//! the only place that holds signals.
//!
//! Fills → round trips pairing (fe `trade-pairing.ts`) and run analytics
//! (fe `analytics.ts`) live in `honba_api_wasm::{pairing, analytics}`; the
//! shared civil-date helpers in `honba_api_wasm::calendar`.
//!
//! Port map:
//! - `defaults.rs` — slider/select constants + fill-model descriptors
//!   (fe `controls.ts` constant blocks);
//! - `controls.rs` — control model → `BacktestRequest` planning
//!   (fe `controls.ts`);
//! - `chart_data.rs` (P2) — bars → chart series, resampling, preview
//!   (fe `chart-data.ts`, `bars.ts`);
//! - `persistence.rs` (P2) — `localStorage` handoff (fe `persistence.ts`);
//! - `ops.rs` (P3/P5 helpers) — run-mode/trust badges + drawdown stats
//!   (fe `ops.ts`);
//! - `selection.rs` (P4) — strategy graph shape, `controls_for_graph`,
//!   `universe_string` (fe `select.ts`, minus the studio port's parts);
//! - `run.rs` (P4) — run state fold + submit/poll loop (fe `run.ts`);
//! - `journal.rs` (P4) — journal fetch fold (fe `journal.ts`);
//! - `log.rs` (P4) — run-event fold (fe `events.ts`);
//! - `manifest.rs` (P4) — graph → compile manifest + compile seam
//!   (fe algodesigner `manifest.ts`/`compile.ts`, the run-facing slice).
//! - `chart_series.rs` (P6) — graph → chart indicator specs (fe
//!   `chart-series.ts`);
//! - `catalogue.rs` (P6) — indicator node catalogue (fe `catalogue.ts`);
//! - `engine/` (P7) — replay engine: `mod.rs` config + status machine,
//!   `indicators.rs` series calculator, `evaluator.rs` condition and
//!   confluence evaluation, `controller.rs` tick/execution state (fe
//!   `core/simulator/engine/*`, portfolio adapter out of port). The store
//!   slice wired the controller into `stores/replay_store.rs` (generation-bump
//!   reactivity, token-guarded frame driver, chart dataset handoff through
//!   a callback prop) and the page grew the `signals`/`portfolio` dock
//!   tabs plus the replay toolbar (`replay_controls.rs`). Still pending:
//!   volume pane, indicator overlays and trade markers — the chart-crate
//!   bridge routes (`setMarkers`, engine-tier `addIndicator`, `SeriesOptions`
//!   panes) are the parallel integration's territory.
//!
//! Presentation (port phase P5, not purged kernels — these may hold signals):
//! - `stores/simulator_store.rs` — the reactive store (over the
//!   `controls_store`, `run_store` and `replay_store` slices), dock tab
//!   strip and execute wiring; `run_store.rs` holds the real `ApiClient`
//!   seams (fe `store/simulator-store.ts`);
//! - `simulator_page.rs` — run toolbar, chart arena (P6/P7: the
//!   replay transport + `signals | portfolio` dock tabs) with the full
//!   `overview | journal | signals | portfolio | heatmap | validation |
//!   logs | payload` strip (fe `SimulatorView` results surface);
//! - `components/*` — metrics ribbon, journal table + CSV,
//!   monthly heatmap, validation badges, run-log/SSE drawer, equity card
//!   (P6: `equity_card`, realised curve + labelled benchmark overlay)
//!   (fe `features/simulator/components/*`, phase-8 shapes).

pub mod bars;
pub mod catalogue;
pub mod chart_data;
pub mod chart_series;
pub mod controls;
pub mod defaults;
pub mod engine;
pub mod journal;
pub mod log;
pub mod manifest;
pub mod ops;
pub mod persistence;
pub mod run;
pub mod selection;

/// Test-only executor for the injected client/clock futures: a `block_on`
/// with **no wakeup scheduling** — it spins `poll` until an immediately-ready
/// future resolves. That covers every test double in `kernel/` (their futures
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
