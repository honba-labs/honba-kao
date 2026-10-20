//! Simulator run slice — the remote run lifecycle: compile, REST
//! submit/poll/abort, the journal load and the run status the toolbar, logs
//! and results surface read. Owned by
//! [`super::simulator_store::SimulatorStore`].
//!
//! Lifecycle honesty carried over from fe + the design's generation counter:
//! - a fresh attempt wipes the previous run's journal first (stale markers
//!   never outlive the run they describe);
//! - the blocked path **sends nothing** and logs `Blocked: …`;
//! - the journal answer is accepted only while no newer fold owns the store
//!   (generation counter instead of fe's runId compare, so `reset`
//!   also invalidates in-flight loads);
//! - a poll cap folds to `failed` — the UI never hangs on "running".

use honba_api_wasm::pairing::{RoundTrip, pair_trades};
use honba_kao_client::ClientError;
use honba_kao_client::wire::{
    BacktestRequest, BacktestResponse, BarsResponse, CompiledStrategy, TradesResponse,
};
use leptos::prelude::*;
use serde_json::Value;

use crate::sim::controls::SimulationRequestPlan;
use crate::sim::journal::{
    JournalClient, JournalState, apply_journal_loading, initial_journal_state, load_journal,
};
use crate::sim::manifest::{StrategyCompiler, compile_strategy};
use crate::sim::run::{
    Clock, RunAbort, RunClient, RunOptions, SimRunState, SimRunStatus, initial_run_state,
    run_backtest,
};
use crate::sim::selection::StrategyGraphShape;

// ---------------------------------------------------------------------------
// The wasm clock (fe's `setTimeout` default)
// ---------------------------------------------------------------------------

/// `setTimeout`-backed implementation of fe's default injected `sleep`.
#[derive(Debug, Default)]
pub struct BrowserClock;

#[cfg(target_arch = "wasm32")]
impl Clock for BrowserClock {
    fn sleep<'a>(&'a self, ms: u64) -> crate::sim::run::ClientFut<'a, ()> {
        Box::pin(async move {
            use wasm_bindgen::JsCast as _;
            use wasm_bindgen::closure::Closure as WasmClosure;

            // `setTimeout(resolve, ms)` by hand: the promise's executor
            // schedules a `Window` timeout whose fire-once callback resolves
            // it. `forget` releases the callback only after the timer holds
            // its reference — a fire-once timer is kept alive by the queue,
            // so the usual leak warning does not apply to a resolvable.
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                let callback = {
                    let resolve = resolve.clone();
                    WasmClosure::once(move || {
                        let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
                    })
                };
                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        ms as i32,
                    );
                    callback.forget();
                } else {
                    // No window (an odd embedding): resolve immediately so
                    // the loop never stalls silently.
                    drop(callback);
                    let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
                }
            });
            let _: wasm_bindgen::JsValue = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .unwrap_or_default();
        })
    }
}

// ---------------------------------------------------------------------------
// The slice
// ---------------------------------------------------------------------------

/// Run lifecycle signals plus the paired round trips derived from the journal.
#[derive(Clone, Copy)]
pub struct RunStore {
    /// The run lifecycle state (`run`).
    pub state: RwSignal<SimRunState>,
    /// The journal fetch state (`journal`).
    pub journal: RwSignal<JournalState>,
    /// The last built request plan (`lastPlan`).
    pub last_plan: RwSignal<Option<SimulationRequestPlan>>,
    /// Bumped by every new run/reset; the journal guard's ownership token.
    /// fe compared run ids after the fact; the counter rejects in-flight
    /// loads of superseded runs and resets alike.
    pub run_generation: RwSignal<u64>,
    /// The journal fills paired into round trips (fe paired at read time in
    /// the chart; the memo is the single source the chart, journal table and
    /// analytics agree on).
    pub trips: Memo<Vec<RoundTrip>>,
}

impl RunStore {
    /// Idle run + journal, no plan submitted yet.
    pub(super) fn new() -> Self {
        let journal = RwSignal::new(initial_journal_state());
        Self {
            state: RwSignal::new(initial_run_state()),
            journal,
            last_plan: RwSignal::new(None),
            run_generation: RwSignal::new(0u64),
            trips: Memo::new(move |_| pair_trades(&journal.get().trades).trips),
        }
    }

    /// Everything the last built attempt submitted (`lastPlan`); `None` until
    /// an execute ran (fe's store kept the same distinction between the
    /// live `plan()` and the persisted `lastPlan`).
    pub fn last_plan(&self) -> Option<SimulationRequestPlan> {
        self.last_plan.get_untracked()
    }

    /// Live guard the tests and store assertions use: is the journal keyed to
    /// this generation's run?
    pub fn generation(&self) -> u64 {
        self.run_generation.get_untracked()
    }

    /// Submits `plan` for `graph` (fe `execute`), the ported lifecycle:
    ///
    /// wipe journal → set plan → blocked path (`Blocked: …` logs, no
    /// request) → running/compiling → compile the manifest → run
    /// (`strategy` = the server's **compiled content id** — no spelling
    /// hacks, W2) → poll cap → journal load under the generation guard.
    ///
    /// Returns `true` once the attempt reached the backtest route — the
    /// caller then persists the selection; the blocked and compile-failure
    /// paths return `false`.
    pub async fn execute<C>(
        &self,
        client: &C,
        mut options: RunOptions<'_>,
        plan: SimulationRequestPlan,
        graph: &StrategyGraphShape,
    ) -> bool
    where
        C: StrategyCompiler + RunClient + JournalClient,
    {
        self.last_plan.set(Some(plan.clone()));
        self.journal.set(initial_journal_state());
        self.run_generation.update(|generation| *generation += 1);
        let generation = self.run_generation.get_untracked();

        // The blocked path never touches the wire.
        if !plan.issues.is_empty() {
            let mut run = initial_run_state();
            run.status = SimRunStatus::Failed;
            run.error = Some(plan.issues.join(" "));
            run.logs = plan
                .issues
                .iter()
                .map(|issue| format!("Blocked: {issue}"))
                .collect();
            self.state.set(run);
            return false;
        }

        let mut running = initial_run_state();
        running.status = SimRunStatus::Running;
        running.logs = vec!["Compiling the strategy…".to_string()];
        self.state.set(running);

        let compiled = compile_strategy(client, graph).await;
        if let Some(error) = compiled.error {
            let mut run = initial_run_state();
            run.status = SimRunStatus::Failed;
            run.error = Some(format!("Compile failed: {error}"));
            run.logs = vec![format!("Compile failed: {error}")];
            self.state.set(run);
            return false;
        }

        let mut request = plan.request.clone();
        request.strategy = Some(compiled.strategy_id.clone());

        options.on_update = {
            let run = self.state;
            Some(Box::new(move |state: &SimRunState| run.set(state.clone())))
        };
        let final_state = run_backtest(client, &request, &mut options).await;
        self.state.set(final_state.clone());

        // Journal fills reach the chart only for a completed run, and only
        // while no newer fold owns the store.
        if final_state.status == SimRunStatus::Completed && final_state.run_id.is_some() {
            let run_id = final_state.run_id.clone().unwrap_or_default();
            self.journal
                .set(apply_journal_loading(&initial_journal_state(), &run_id));
            let journal = load_journal(client, &run_id).await;
            if self.run_generation.get_untracked() == generation {
                self.journal.set(journal);
            }
        }

        true
    }

    /// Clears the lifecycle (`resetRun`); the generation bump also
    /// invalidates any journal load still in flight.
    pub fn reset(&self) {
        self.run_generation.update(|generation| *generation += 1);
        self.state.set(initial_run_state());
        self.journal.set(initial_journal_state());
        self.last_plan.set(None);
    }
}

// ---------------------------------------------------------------------------
// The real wire client + execute cadence (P5 wires these into the page)
// ---------------------------------------------------------------------------

/// Millis between polls (fe `run.ts` default: 1s ⇒ ~2 min across 120).
pub const POLL_INTERVAL_MS: u64 = 1_000;

/// Polls before the fold reports `failed` (fe `run.ts` default).
pub const MAX_POLLS: u32 = 120;

impl StrategyCompiler for honba_kao_client::ApiClient {
    fn post_strategies_manifest<'a>(
        &'a self,
        manifest: &'a Value,
    ) -> crate::sim::run::ClientFut<'a, Result<CompiledStrategy, ClientError>> {
        Box::pin(self.post_strategies(manifest))
    }
}

impl RunClient for honba_kao_client::ApiClient {
    fn post_backtests<'a>(
        &'a self,
        request: &'a BacktestRequest,
    ) -> crate::sim::run::ClientFut<'a, Result<BacktestResponse, ClientError>> {
        Box::pin(self.post_backtests(request))
    }

    fn get_backtest<'a>(
        &'a self,
        id: &'a str,
    ) -> crate::sim::run::ClientFut<'a, Result<BacktestResponse, ClientError>> {
        Box::pin(self.get_backtest(id))
    }
}

impl JournalClient for honba_kao_client::ApiClient {
    fn get_backtest_journal<'a>(
        &'a self,
        id: &'a str,
    ) -> crate::sim::run::ClientFut<'a, Result<TradesResponse, ClientError>> {
        Box::pin(self.get_backtest_journal(id))
    }
}

impl crate::sim::bars::BarsSeam for honba_kao_client::ApiClient {
    fn get_bars<'a>(
        &'a self,
        id: &'a str,
        tf: Option<&'a str>,
        from: Option<&'a str>,
        to: Option<&'a str>,
    ) -> crate::sim::run::ClientFut<'a, Result<BarsResponse, ClientError>> {
        Box::pin(self.get_bars(id, tf, from, to))
    }
}

/// A fresh `RunOptions` for the page's execute wiring — fe's default cadence,
/// no update hook (the store installs its own), abort flag supplied by the
/// page so `on_cleanup` can stop an in-flight run.
pub fn run_options<'a>(clock: &'a dyn Clock, abort: RunAbort) -> RunOptions<'a> {
    RunOptions {
        interval_ms: POLL_INTERVAL_MS,
        max_polls: MAX_POLLS,
        abort: Some(abort),
        clock,
        on_update: None,
    }
}
