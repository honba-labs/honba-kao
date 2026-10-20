//! The simulator store (HBD-2026-DES-005 P4).
//!
//! The Leptos port of honba-fe `features/simulator/store/simulator-store.ts`.
//! Owns the active strategy graph, the control panel, and the run lifecycle:
//! selection → plan → (blocked?) → compile → submit/poll → journal. The pure
//! logic lives in `sim/*`; this slice is the thin signal binding the pages
//! and components read.
//!
//! fe's zustand store kept every field in one bag; per the design doc's perf
//! note ("don't replicate the monolith store read") the state is a set of
//! **granular signals**, wrapped with `Memo`s for the plan, the paired round
//! trips and the analytics bundle — the chart, ribbon and journal re-render
//! independently.
//!
//! Lifecycle honesty carried over from fe + the design's generation counter:
//! - a fresh attempt wipes the previous run's journal first (stale markers
//!   never outlive the run they describe);
//! - the blocked path **sends nothing** and logs `Blocked: …`;
//! - the journal answer is accepted only while no newer fold owns the store
//!   (generation counter instead of fe's runId compare, so `reset_run`
//!   also invalidates in-flight loads);
//! - a poll cap folds to `failed` — the UI never hangs on "running".
//!
//! The strategy-studio-dependent parts of fe's `hydrate` (URL slug, library
//! manifests, Designer hand-off) are **not** ported yet (P8): a reload
//! restores the persisted name/origin/controls over the built-in template
//! graph, the one selection this build can resolve.
//!
//! Manual serve checklist (the P7 acceptance note, for `make serve`,
//! browser-only by construction — the native build never renders the page):
//! 1. Open `/simulator` with the API up: the chart badge reads `Served
//!    bars`; the replay toolbar shows `Play` enabled once the dataset
//!    lands, the HUD reads `Waiting for bars…` and the book row reads
//!    `Flat (No open position)`.
//! 2. `Play` at `5x`: candles advance bar-by-bar (settled prefix plus the
//!    forming candle), the signal pills light on the close phase, the
//!    confluence pill trails the last close. Pausing freezes; `Step`
//!    advances one tick from a frozen transport.
//! 3. Drag the scrubber while paused: the chart lands **including** the
//!    landing bar (fe's `slice(0, bounded + 1)` emission); while playing
//!    the timer keeps running from the landing bar.
//! 4. `Run to End` completes: equity/realized tally in the toolbar row,
//!    the `portfolio` tab lists the engine's round trips (engine books —
//!    no account rows), the `signals` tab shows the last confluence
//!    detail, and `Play` reads `Replay` and restarts from bar 0.
//! 5. Kill the API and change the window (or reload): the badge flips to
//!    `Preview bars`, the published dataset changes, and the books reset —
//!    a re-init wipes trips/position, never silently continuing.
//! 6. Theme switch recolors the toolbar/panels through tokens only.
//!
//! Replay wiring (P7 store-integration slice): the
//! [`ReplayController`] stays plain state — no signals inside it, no timers
//! — and its owner here bumps a generation [`RwSignal`] after every
//! mutation, matching fe's hook-shaped `setState` notify. fe re-initialised
//! the engine from the *chart's* bars on an `initKey` change; here the
//! chart container publishes its loaded dataset through the store's
//! `engine_bars` signal (via a callback prop), so the replay shares
//! the chart's single fetch. The frame driver is one `spawn_local` loop
//! token-guarded against re-arm (fe's `setInterval`); the driver bumps the
//! generation per frame — the dock tabs, the HUD pills and the visible-bars
//! chart swap all read it.
//!

use std::sync::Arc;

use honba_kao_client::ClientError;
use honba_kao_client::wire::{
    BacktestMetrics, BacktestRequest, BacktestResponse, BarsResponse, CompiledStrategy,
    TradesResponse,
};
use leptos::prelude::*;
use serde_json::Value;

use crate::sim::analytics::{
    AnalyticsInput, EquitySeries, JournalRow, MonthlyHeatmap, RibbonMetric, RoundTripStats,
    RunWindow, derive_ribbon_metrics, equity_series, journal_rows, monthly_heatmap,
    round_trip_stats,
};
use crate::sim::bars::BarsState;
use crate::sim::chart_data::ChartBar;
use crate::sim::controls::{
    PeriodPreset, PlanOptions, SimulationRequestPlan, SimulatorControls, Today, default_controls,
    plan_request,
};
use crate::sim::engine::controller::{ReplayController, SimulationEngineState};
use crate::sim::engine::{DEFAULT_ACCOUNT_ID, FillModel, ReplayConfig, ReplayStatus};
use crate::sim::journal::{
    JournalClient, JournalState, apply_journal_loading, initial_journal_state, load_journal,
};
use crate::sim::manifest::{StrategyCompiler, compile_strategy};
use crate::sim::pairing::{RoundTrip, pair_trades};
use crate::sim::persistence::{
    LayoutStorage, SimulatorPersisted, load_persisted, persist_selection,
};
use crate::sim::run::{
    Clock, RunAbort, RunClient, RunOptions, SimRunState, SimRunStatus, initial_run_state,
    run_backtest,
};
use crate::sim::selection::{
    SelectionOrigin, StrategyGraphShape, builtin_template, controls_for_graph, universe_string,
};

// ---------------------------------------------------------------------------
// The analytics bundle (P5's results surface reads this)
// ---------------------------------------------------------------------------

/// Everything the results surface reads, derived in one memo
/// (`analyticsBundle`). The benchmark overlay stays `None` until P6 wires
/// the bar route — an absent overlay is labelled, never fabricated.
#[derive(Clone, Debug, PartialEq)]
pub struct SimAnalytics {
    /// The run payload's wire metrics, when the run reported them.
    pub metrics: Option<BacktestMetrics>,
    /// The resolved run window (from the last plan), when known.
    pub window: Option<RunWindow>,
    /// The ten ribbon metrics, in fe's exact key order.
    pub ribbon: Vec<RibbonMetric>,
    /// Realised equity.
    pub equity_series: EquitySeries,
    /// Month × year heatmap (IST).
    pub heatmap: MonthlyHeatmap,
    /// Journal table rows (+ the CSV export's data, design wart W5).
    pub rows: Vec<JournalRow>,
    /// Trade-level statistics.
    pub stats: RoundTripStats,
    /// Benchmark overlay values, index-aligned to `equity_series.equity`;
    /// `None` until P6 wires the benchmark bars.
    pub benchmark: Option<Vec<f64>>,
}

// ---------------------------------------------------------------------------
// Controls patch (fe's `Partial<SimulatorControls>`)
// ---------------------------------------------------------------------------

/// A partial control update (`setControls(patch)`) — every field optional,
/// `None` leaves the current value in place.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlsPatch {
    /// Trading symbol.
    pub symbol: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Benchmark label.
    pub benchmark: Option<String>,
    /// Period preset.
    pub period: Option<PeriodPreset>,
    /// Bar spec (`5m`).
    pub timeframe: Option<String>,
    /// Initial capital.
    pub capital: Option<f64>,
    /// Flat commission per fill.
    pub commission: Option<f64>,
    /// Slippage, basis points.
    pub slippage: Option<f64>,
    /// Fill model id.
    pub fill_model: Option<String>,
    /// Custom start — `Some(None)` clears the field.
    pub start: Option<Option<String>>,
    /// Custom end — `Some(None)` clears the field.
    pub end: Option<Option<String>>,
}

impl ControlsPatch {
    /// Applies the present fields over `controls` (fe's `{ …current, …patch }`).
    pub fn apply(&self, controls: &mut SimulatorControls) {
        if let Some(value) = &self.symbol {
            controls.symbol = value.clone();
        }
        if let Some(value) = &self.exchange {
            controls.exchange = value.clone();
        }
        if let Some(value) = &self.benchmark {
            controls.benchmark = value.clone();
        }
        if let Some(value) = self.period {
            controls.period = value;
        }
        if let Some(value) = &self.timeframe {
            controls.timeframe = value.clone();
        }
        if let Some(value) = self.capital {
            controls.capital = value;
        }
        if let Some(value) = self.commission {
            controls.commission = value;
        }
        if let Some(value) = self.slippage {
            controls.slippage = value;
        }
        if let Some(value) = &self.fill_model {
            controls.fill_model = value.clone();
        }
        if let Some(value) = &self.start {
            controls.start = value.clone();
        }
        if let Some(value) = &self.end {
            controls.end = value.clone();
        }
    }
}

// ---------------------------------------------------------------------------
// The wasm clock + today helper (fe's `setTimeout` defaults)
// ---------------------------------------------------------------------------

/// `setTimeout`-backed implementation of fe's default injected `sleep`.
#[derive(Debug, Default)]
pub struct BrowserClock;

/// The current UTC calendar day from the browser's clock (fe read
/// `new Date()`'s UTC fields inside `toBacktestRequest`).
#[cfg(target_arch = "wasm32")]
pub fn utc_today() -> Today {
    let date = js_sys::Date::new_0();
    Today::new(
        date.get_utc_full_year() as i32,
        date.get_utc_month() + 1,
        date.get_utc_date(),
    )
}

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
// The store
// ---------------------------------------------------------------------------

/// The bottom dock's tab strip (fe's `dockTab` union, shaped like the
/// screener's `DetailTab` enum). The `signals`/`portfolio` tabs fe also
/// carried arrive with the replay engine (P7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SimTab {
    /// Metrics, header, actions.
    #[default]
    Overview,
    /// The trade journal table.
    Journal,
    /// The replay engine's confluence detail (P7; fe's `signals` dock).
    Signals,
    /// The sandbox-local book — engine trips + open position (P7; fe's
    /// `portfolio` dock).
    Portfolio,
    /// The month × year heatmap.
    Heatmap,
    /// Trust badges + not-modelled assumptions.
    Validation,
    /// Run log + SSE event tail.
    Logs,
    /// The raw request payload.
    Payload,
}

impl SimTab {
    /// fe's spelling, shown in the strip.
    pub const fn as_str(self) -> &'static str {
        match self {
            SimTab::Overview => "overview",
            SimTab::Journal => "journal",
            SimTab::Signals => "signals",
            SimTab::Portfolio => "portfolio",
            SimTab::Heatmap => "heatmap",
            SimTab::Validation => "validation",
            SimTab::Logs => "logs",
            SimTab::Payload => "payload",
        }
    }
}

/// The simulator store: granular signals + derived memos, methods mirror fe's
/// `SimulatorState`.
///
/// Construct inside a reactive owner (a component body or a test's runtime);
/// the memos are rooted at construction. Cloning shares the same signals —
/// the pages hand the store down and the execute future keeps a clone.
#[derive(Clone)]
pub struct SimulatorStore {
    /// The active strategy graph (`strategy`).
    pub strategy: RwSignal<StrategyGraphShape>,
    /// Its display name (`strategyName`).
    pub strategy_name: RwSignal<String>,
    /// Where the selection came from (`origin`).
    pub origin: RwSignal<SelectionOrigin>,
    /// The injected UTC day the date math runs on (fe computed `new Date()`
    /// per plan build; a signal keeps the memos correct across midnight).
    pub today: RwSignal<Today>,
    /// The control panel state (`controls`).
    pub controls: RwSignal<SimulatorControls>,
    /// The run lifecycle state (`run`).
    pub run: RwSignal<SimRunState>,
    /// The journal fetch state (`journal`).
    pub journal: RwSignal<JournalState>,
    /// The last built request plan (`lastPlan`).
    pub last_plan: RwSignal<Option<SimulationRequestPlan>>,
    /// Bumped by every new run/reset; the journal guard's ownership token.
    /// fe compared run ids after the fact; the counter rejects in-flight
    /// loads of superseded runs and resets alike.
    pub run_generation: RwSignal<u64>,
    /// The bottom dock's active tab (`dockTab`).
    pub tab: RwSignal<SimTab>,
    /// Best-effort handoff storage (fe: `localStorage`); `None` in tests.
    pub storage: Option<Arc<dyn LayoutStorage>>,

    /// The memoised request plan for the current controls (fe's `plan()` was
    /// rebuilt on every read; a memo makes `lastPlan`/payload previews share
    /// one allocation per edit).
    pub plan: Memo<SimulationRequestPlan>,
    /// The journal fills paired into round trips (fe paired at read time in
    /// the chart; the memo is the single source the chart, journal table and
    /// analytics agree on).
    pub trips: Memo<Vec<RoundTrip>>,
    /// The analytics bundle for the results surface.
    pub analytics_bundle: Memo<SimAnalytics>,

    /// The P7 replay engine — plain state held once, mutated in place
    /// (fe kept the controller in a hook; here the store owns it).
    pub replay: StoredValue<ReplayController>,
    /// Bumped by every replay mutation. The dock's engine-state memo and
    /// each panel read track this one integer (fe's `setState` notify).
    pub replay_generation: RwSignal<u64>,
    /// Bumped whenever the frame-driver loop must retire (pause, reset,
    /// init, speed change, unmount); each loop holds its issued token and
    /// exits when the signal moves past it.
    pub replay_driver: RwSignal<u64>,
    /// The dataset the chart container loaded (fe read the dataLayer bars
    /// the chart already held; the replay shares them without a second
    /// fetch). The chart publishes through a callback prop.
    pub engine_bars: RwSignal<BarsState>,
    /// The controller's visible-bars slice (fe's `onBarChange` emission);
    /// `None` while idle, when the chart shows the full loaded dataset.
    pub replay_visible: RwSignal<Option<Vec<ChartBar>>>,
}

impl SimulatorStore {
    /// A fresh store: template selection, default controls, idle run +
    /// journal. fe's `initialState()` ran `resolveSelection({})`, whose
    /// template fallback is all this build resolves without the library.
    pub fn new(today: Today, storage: Option<Arc<dyn LayoutStorage>>) -> Self {
        let (graph, name) = builtin_template();
        let strategy = RwSignal::new(graph);
        let strategy_name = RwSignal::new(name.to_string());
        let origin = RwSignal::new(SelectionOrigin::Template);
        let today_signal = RwSignal::new(today);
        let controls = RwSignal::new(default_controls());
        let run = RwSignal::new(initial_run_state());
        let journal = RwSignal::new(initial_journal_state());
        let last_plan = RwSignal::new(None);
        let run_generation = RwSignal::new(0u64);
        let tab = RwSignal::new(SimTab::Overview);

        let replay = StoredValue::new(ReplayController::default());
        let replay_generation = RwSignal::new(0u64);
        let replay_driver = RwSignal::new(0u64);
        let engine_bars = RwSignal::new(BarsState::initial());
        let replay_visible = RwSignal::new(None);

        let plan_memo = {
            Memo::new(move |_| {
                let graph = strategy.get();
                let name = strategy_name.get();
                let controls = controls.get();
                let today = today_signal.get();
                plan_request(
                    &controls,
                    &PlanOptions {
                        strategy: &name,
                        universe: Some(&universe_string(&graph, &controls)),
                        seed: None,
                        today,
                    },
                )
            })
        };

        let trips_memo = { Memo::new(move |_| pair_trades(&journal.get().trades).trips) };

        let analytics_memo = {
            Memo::new(move |_| {
                let run = run.get();
                let trips = trips_memo.get();
                let last_plan = last_plan.get();
                let capital = controls.get().capital;
                let metrics = run
                    .response
                    .as_ref()
                    .and_then(|response| response.metrics.clone());
                let window: Option<RunWindow> =
                    last_plan
                        .as_ref()
                        .map(|plan: &SimulationRequestPlan| RunWindow {
                            start: plan.range.0.clone(),
                            end: plan.range.1.clone(),
                        });
                let input = AnalyticsInput {
                    metrics: metrics.as_ref(),
                    trips: &trips,
                    window: window.as_ref(),
                    initial_capital: Some(capital),
                };
                SimAnalytics {
                    metrics: metrics.clone(),
                    window: window.clone(),
                    ribbon: derive_ribbon_metrics(&input),
                    equity_series: equity_series(&input),
                    heatmap: monthly_heatmap(&input),
                    rows: journal_rows(&trips),
                    stats: round_trip_stats(&trips),
                    benchmark: None,
                }
            })
        };

        Self {
            strategy,
            strategy_name,
            origin,
            today: today_signal,
            controls,
            run,
            journal,
            last_plan,
            run_generation,
            tab,
            storage,
            plan: plan_memo,
            trips: trips_memo,
            analytics_bundle: analytics_memo,
            replay,
            replay_generation,
            replay_driver,
            engine_bars,
            replay_visible,
        }
    }

    /// Boots the store from whatever persistence offers (`hydrate`).
    ///
    /// fe's branches over `?strategy=` slugs, Designer hand-offs and library
    /// manifests belong to the studio port (P8). Here: a persisted selection
    /// re-installs its name/origin/controls over the template graph; nothing
    /// available means the template stays untouched. fe validated the
    /// persisted name against the library before trusting it — deferred with
    /// the library itself.
    pub fn hydrate(&self) {
        let Some(storage) = self.storage.as_ref() else {
            return;
        };
        let Some(persisted) = load_persisted(storage.as_ref()) else {
            return;
        };
        let (graph, _) = builtin_template();
        self.strategy.set(graph);
        self.strategy_name.set(persisted.strategy_name.clone());
        self.origin.set(persisted.source);
        self.controls.set(persisted.controls.clone());
    }

    /// Patches the control panel (`setControls`) — the memos re-derive.
    pub fn set_controls(&self, patch: ControlsPatch) {
        self.controls.update(|controls| patch.apply(controls));
    }

    /// Adopts a new selection (`select`): the graph's universe/timeframe
    /// pre-fill the panel, the rest of the controls stay, and the selection
    /// persists — a strategy switch never resets the operator's window.
    pub fn select(&self, graph: StrategyGraphShape, name: &str, origin: SelectionOrigin) {
        let controls = {
            let current = self.controls.get_untracked();
            controls_for_graph(&graph, &current)
        };
        self.strategy.set(graph);
        self.strategy_name.set(name.to_string());
        self.origin.set(origin);
        self.controls.set(controls);
        self.persist();
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

    /// Submits the current plan (fe `execute`), the ported lifecycle:
    ///
    /// wipe journal → set plan → blocked path (`Blocked: …` logs, no
    /// request) → running/compiling → compile the manifest → run
    /// (`strategy` = the server's **compiled content id** — no spelling
    /// hacks, W2) → poll cap → journal load under the generation guard →
    /// persist the selection.
    pub async fn execute<C>(&self, client: &C, mut options: RunOptions<'_>)
    where
        C: StrategyCompiler + RunClient + JournalClient,
    {
        let plan = self.plan.get_untracked();
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
            self.run.set(run);
            return;
        }

        let mut running = initial_run_state();
        running.status = SimRunStatus::Running;
        running.logs = vec!["Compiling the strategy…".to_string()];
        self.run.set(running);

        let graph = self.strategy.get_untracked();
        let compiled = compile_strategy(client, &graph).await;
        if let Some(error) = compiled.error {
            let mut run = initial_run_state();
            run.status = SimRunStatus::Failed;
            run.error = Some(format!("Compile failed: {error}"));
            run.logs = vec![format!("Compile failed: {error}")];
            self.run.set(run);
            return;
        }

        let mut request = plan.request.clone();
        request.strategy = Some(compiled.strategy_id.clone());

        options.on_update = {
            let run = self.run;
            Some(Box::new(move |state: &SimRunState| run.set(state.clone())))
        };
        let final_state = run_backtest(client, &request, &mut options).await;
        self.run.set(final_state.clone());

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

        self.persist();
    }

    /// Clears the lifecycle (`resetRun`); the generation bump also
    /// invalidates any journal load still in flight.
    pub fn reset_run(&self) {
        self.run_generation.update(|generation| *generation += 1);
        self.run.set(initial_run_state());
        self.journal.set(initial_journal_state());
        self.last_plan.set(None);
    }

    // -- Replay wiring (P7 store-integration slice) -------------------------
    // The controller stays signal-free (P7 contract); every method below
    // mutates it in place, then syncs the reactive surface: the visible
    // slice (fe's `onBarChange`) and the generation bump (fe's `setState`).

    /// Snapshots `ReplayController::state()` without cloning the controller
    /// (`try_update_value` mutates in place; nothing is written, so this is
    /// a read that happens to borrow through the write guard). Call inside
    /// a memo tracking `replay_generation`.
    pub fn replay_state(&self) -> SimulationEngineState {
        self.replay
            .try_update_value(|controller| controller.state())
            .expect("the replay controller outlives its store")
    }

    /// fe's `initKey` re-init: a new dataset (the chart's publish) plus a
    /// fresh config from the current controls — fe re-initialised on
    /// `symbol|exchange|timeframe|bars|capital|account|fillModel|
    /// commission|slippage`; here the instrument split, the capital and the
    /// fill settings ride `controls`, so one store method covers them. The
    /// speed survives re-initialisation (fe `init` never touched it).
    pub fn init_replay(&self, bars: &[ChartBar], instrument_id: &str) {
        if bars.is_empty() {
            return;
        }
        let controls = self.controls.get_untracked();
        let (symbol, exchange) = match instrument_id.rsplit_once('.') {
            Some((symbol, exchange)) if !symbol.is_empty() && !exchange.is_empty() => {
                (symbol.to_string(), exchange.to_string())
            }
            // The universe id should always be `SYMBOL.EXCHANGE`; fall back
            // to the controls' own fields rather than inventing a symbol.
            _ => (controls.symbol.clone(), controls.exchange.clone()),
        };
        let config = ReplayConfig {
            symbol,
            exchange,
            initial_capital: controls.capital,
            account_id: DEFAULT_ACCOUNT_ID.to_string(),
            fill_model: FillModel::from_name(&controls.fill_model),
            commission: controls.commission,
            slippage_bps: controls.slippage,
            // fe's `init` left `speed` alone (it was constructed once with
            // the hook); carry the clamped value across re-inits.
            speed: self
                .replay
                .try_update_value(|controller| controller.speed())
                .unwrap_or(1.0),
        };
        let graph = self.strategy.get_untracked();
        self.replay_driver.update(|token| *token += 1);
        self.replay
            .update_value(|controller| controller.init(bars, &graph, config));
        self.after_replay_mutation();
    }

    /// fe `play` — reset-when-exhausted semantics live in the controller;
    /// the frame-driver loop is re-armed here (the controller owns no
    /// timers, the app drives `run_frame`).
    pub fn replay_play(&self) {
        self.replay.update_value(|controller| controller.play());
        self.replay_spawn_driver();
        self.after_replay_mutation();
    }

    /// fe `pause`.
    pub fn replay_pause(&self) {
        // Retire the loop before the status flips, so no queued frame
        // ticks behind the pause.
        self.replay_driver.update(|token| *token += 1);
        self.replay.update_value(|controller| controller.pause());
        self.after_replay_mutation();
    }

    /// fe `step` — one tick; pausing (and retiring the loop) is the
    /// controller's own semantics, this only keeps the driver in step.
    pub fn replay_step(&self) {
        let playing = self
            .replay
            .try_update_value(|controller| {
                controller.step();
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if !playing {
            self.replay_driver.update(|token| *token += 1);
        }
        self.after_replay_mutation();
    }

    /// fe `reset`.
    pub fn replay_reset(&self) {
        self.replay_driver.update(|token| *token += 1);
        self.replay.update_value(|controller| controller.reset());
        self.after_replay_mutation();
    }

    /// fe `instantRunToEnd` — sweep every bar at its close phase.
    pub fn replay_instant(&self) {
        self.replay_driver.update(|token| *token += 1);
        self.replay
            .update_value(|controller| controller.instant_run_to_end());
        self.after_replay_mutation();
    }

    /// fe `setSpeed` — while playing the interval changes, so the timer is
    /// re-armed at the new cadence (fe stopped and restarted its timer).
    pub fn replay_set_speed(&self, speed: f64) {
        let playing = self
            .replay
            .try_update_value(|controller| {
                controller.set_speed(speed);
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if playing {
            self.replay_spawn_driver();
        }
        self.after_replay_mutation();
    }

    /// fe `jumpToBar` — scrub across the dataset. While playing the timer
    /// continues from the landing bar (fe's jump never stopped its timer).
    pub fn replay_jump(&self, index: usize) {
        let playing = self
            .replay
            .try_update_value(|controller| {
                controller.jump_to_bar(index);
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if !playing {
            self.replay_driver.update(|token| *token += 1);
        }
        self.after_replay_mutation();
    }

    /// fe's timer body: one frame of ticks, and the books' visible slice
    /// re-cut (called only by the driver loop).
    pub fn replay_frame(&self) {
        self.replay
            .update_value(|controller| controller.run_frame());
        self.after_replay_mutation();
    }

    /// Syncs the reactive surface after any controller mutation: the
    /// visible slice (fe's `onBarChange`) and the generation bump (fe's
    /// state notification).
    fn after_replay_mutation(&self) {
        let visible = self
            .replay
            .try_update_value(|controller| controller.visible_bars())
            .flatten();
        self.replay_visible.set(visible);
        self.replay_generation.update(|generation| *generation += 1);
    }

    /// Arms a fresh frame-driver loop: the previous one retires via the
    /// driver token, the new loop waits each frame's `interval_ms` on the
    /// browser event loop (fe's `setInterval(intervalMs)`; the controller
    /// owns no timers per the P7 contract). Native builds never render the
    /// page, so there the loop is a no-op and callers advance by step.
    fn replay_spawn_driver(&self) {
        self.replay_driver.update(|token| *token += 1);
        let token = self.replay_driver.get_untracked();
        #[cfg(target_arch = "wasm32")]
        {
            let store = self.clone();
            let driver = self.replay_driver;
            leptos::task::spawn_local(async move {
                loop {
                    if driver.get_untracked() != token {
                        return;
                    }
                    // fe read `intervalMs` off the live config every frame;
                    // the speed may change between sleeps.
                    let interval = store
                        .replay
                        .try_update_value(|controller| {
                            crate::sim::engine::interval_ms(controller.speed())
                        })
                        .unwrap_or(180);
                    Clock::sleep(&BrowserClock, interval as u64).await;
                    if driver.get_untracked() != token {
                        return;
                    }
                    // A pause could have raced a queued frame even without
                    // a token bump (same token, different world); never
                    // tick a controller that is not playing.
                    if store
                        .replay
                        .try_update_value(|controller| controller.status())
                        != Some(ReplayStatus::Playing)
                    {
                        return;
                    }
                    store.replay_frame();
                    if store
                        .replay
                        .try_update_value(|controller| controller.status())
                        == Some(ReplayStatus::Completed)
                    {
                        return;
                    }
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = token;
        }
    }

    /// Stashes the current selection (fe's `persistSelection`, best-effort
    /// and silent in a storage-less environment).
    fn persist(&self) {
        if let Some(storage) = self.storage.as_ref() {
            let selection = SimulatorPersisted {
                strategy_name: self.strategy_name.get_untracked(),
                source: self.origin.get_untracked(),
                controls: self.controls.get_untracked(),
            };
            persist_selection(&selection, storage.as_ref());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::journal::JournalStatus;
    use crate::sim::run::ClientFut;
    use crate::sim::testing::block_on;
    use honba_kao_client::ClientError;
    use honba_kao_client::wire::{
        BacktestRequest, BacktestResponse, CompiledStrategy, Currency, Exchange, InstrumentId,
        OrderId, RunStatus, Trade, TradesResponse, WireOrderSide, WireUnixNanos,
    };
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// The storage double (fe's localStorage stubs).
    #[derive(Default)]
    struct FakeStorage(Mutex<String>);

    impl LayoutStorage for FakeStorage {
        fn get_item(&self, _key: &str) -> Option<String> {
            let raw = self.0.lock().expect("fake storage lock");
            (!raw.is_empty()).then(|| raw.clone())
        }

        fn set_item(&self, _key: &str, value: &str) {
            *self.0.lock().expect("fake storage lock") = value.to_string();
        }
    }

    /// fe's instant injectable clock.
    struct InstantClock;
    impl Clock for InstantClock {
        fn sleep<'a>(&'a self, _ms: u64) -> ClientFut<'a, ()> {
            Box::pin(async {})
        }
    }

    fn options<'a>() -> RunOptions<'a> {
        static CLOCK: InstantClock = InstantClock;
        RunOptions::new(&CLOCK)
    }

    /// The client double (`mockClient` in fe's tests): one compile answer,
    /// one submit answer, a poll queue, and the journal answer; counters
    /// prove which routes the lifecycle actually touched.
    struct FakeClient {
        compiled: Result<CompiledStrategy, ClientError>,
        submit: Result<BacktestResponse, ClientError>,
        polls: RefCell<VecDeque<BacktestResponse>>,
        journal: Result<TradesResponse, ClientError>,
        before_journal: Option<Box<dyn Fn()>>,
        compile_calls: Cell<u32>,
        submit_calls: Cell<u32>,
        poll_calls: Cell<u32>,
        journal_calls: Cell<u32>,
    }

    impl FakeClient {
        fn never_touches_the_wire() -> Self {
            Self {
                compiled: Err(ClientError::Network("must not be called".to_string())),
                submit: Err(ClientError::Network("must not be called".to_string())),
                polls: RefCell::new(VecDeque::new()),
                journal: Err(ClientError::Network("must not be called".to_string())),
                before_journal: None,
                compile_calls: Cell::new(0),
                submit_calls: Cell::new(0),
                poll_calls: Cell::new(0),
                journal_calls: Cell::new(0),
            }
        }

        fn successful_run() -> Self {
            Self {
                compiled: Ok(compiled_fixture("sha256:abc")),
                submit: Ok(running_response("r1")),
                polls: RefCell::new(vec![completed_response("r1")].into()),
                journal: Ok(TradesResponse {
                    trades: vec![fill(0), fill(10)],
                }),
                before_journal: None,
                compile_calls: Cell::new(0),
                submit_calls: Cell::new(0),
                poll_calls: Cell::new(0),
                journal_calls: Cell::new(0),
            }
        }

        fn stuck_run() -> Self {
            Self {
                compiled: Ok(compiled_fixture("sha256:abc")),
                submit: Ok(running_response("r1")),
                polls: RefCell::new(VecDeque::new()),
                journal: Err(ClientError::Network("must not be called".to_string())),
                before_journal: None,
                compile_calls: Cell::new(0),
                submit_calls: Cell::new(0),
                poll_calls: Cell::new(0),
                journal_calls: Cell::new(0),
            }
        }

        fn failing_compile() -> Self {
            Self {
                compiled: Err(ClientError::Network("broken".to_string())),
                submit: Err(ClientError::Network("must not be called".to_string())),
                polls: RefCell::new(VecDeque::new()),
                journal: Err(ClientError::Network("must not be called".to_string())),
                before_journal: None,
                compile_calls: Cell::new(0),
                submit_calls: Cell::new(0),
                poll_calls: Cell::new(0),
                journal_calls: Cell::new(0),
            }
        }
    }

    impl StrategyCompiler for FakeClient {
        fn post_strategies_manifest<'a>(
            &'a self,
            _manifest: &'a serde_json::Value,
        ) -> ClientFut<'a, Result<CompiledStrategy, ClientError>> {
            self.compile_calls.set(self.compile_calls.get() + 1);
            let answer = match &self.compiled {
                Ok(compiled) => Ok(compiled.clone()),
                Err(error) => Err(error.clone()),
            };
            Box::pin(async move { answer })
        }
    }

    impl RunClient for FakeClient {
        fn post_backtests<'a>(
            &'a self,
            _request: &'a BacktestRequest,
        ) -> ClientFut<'a, Result<BacktestResponse, ClientError>> {
            self.submit_calls.set(self.submit_calls.get() + 1);
            let answer = match &self.submit {
                Ok(response) => Ok(response.clone()),
                Err(error) => Err(error.clone()),
            };
            Box::pin(async move { answer })
        }

        fn get_backtest<'a>(
            &'a self,
            _id: &'a str,
        ) -> ClientFut<'a, Result<BacktestResponse, ClientError>> {
            self.poll_calls.set(self.poll_calls.get() + 1);
            let polled = self
                .polls
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| running_response("r1"));
            Box::pin(async move { Ok(polled) })
        }
    }

    impl JournalClient for FakeClient {
        fn get_backtest_journal<'a>(
            &'a self,
            _id: &'a str,
        ) -> ClientFut<'a, Result<TradesResponse, ClientError>> {
            self.journal_calls.set(self.journal_calls.get() + 1);
            if let Some(hook) = &self.before_journal {
                hook();
            }
            let answer = match &self.journal {
                Ok(response) => Ok(response.clone()),
                Err(error) => Err(error.clone()),
            };
            Box::pin(async move { answer })
        }
    }

    /// The compile answer exactly as the P0 wire fixture carries it.
    fn compiled_fixture(id: &str) -> CompiledStrategy {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "ir": {
                "schema_version": 4,
                "strategy_api_version": "1.0.0",
                "manifest": {
                    "api_version": "1.0.0",
                    "name": "SMA Cross",
                    "source_hash": "sha256:abc",
                    "universe": { "explicit": [ { "symbol": "TCS", "exchange": "NSE" } ] },
                    "subscriptions": {
                        "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ],
                        "quotes": false,
                        "trades": false
                    },
                    "driving_timeframe": { "interval": 5, "aggregation": "minute" },
                    "warmup_bars": 2
                },
                "universe": {
                    "named": null,
                    "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ]
                },
                "subscriptions": {
                    "bars": [ { "symbol": "TCS", "exchange": "NSE" } ],
                    "quotes": [],
                    "trades": []
                },
                "driving_timeframe": { "interval": 5, "aggregation": "minute" },
                "timeframes": [ { "interval": 5, "aggregation": "minute" } ],
                "warmup_bars": 2
            }
        }))
        .expect("compiled strategy decodes")
    }

    fn running_response(run_id: &str) -> BacktestResponse {
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Running,
            metrics: None,
            assumptions: None,
            validation: None,
            error: None,
        }
    }

    fn completed_response(run_id: &str) -> BacktestResponse {
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Completed,
            metrics: Some(honba_kao_client::wire::BacktestMetrics {
                trades: 2,
                net_pnl: 500.0,
                sharpe: 1.2,
                max_drawdown: 0.05,
                total_return: 0.05,
            }),
            assumptions: None,
            validation: None,
            error: None,
        }
    }

    fn fill(minute: i64) -> Trade {
        Trade::new(
            OrderId::new("o"),
            InstrumentId::new("TCS", Exchange::new("NSE")),
            WireOrderSide::Buy,
            10.0,
            100.0,
            Currency::Inr,
            WireUnixNanos::from_u64((1_704_153_600 + minute * 60) as u64 * 1_000_000_000),
            WireUnixNanos::from_u64((1_704_153_600 + minute * 60) as u64 * 1_000_000_000),
        )
    }

    fn valid_controls() -> ControlsPatch {
        ControlsPatch {
            symbol: Some("TCS".to_string()),
            ..Default::default()
        }
    }

    /// Runs the body inside a reactive owner so the store's memos root.
    fn with_store<T>(
        storage: Option<Arc<dyn LayoutStorage>>,
        f: impl FnOnce(&SimulatorStore) -> T,
    ) -> T {
        let owner = leptos::prelude::Owner::new();
        let result = owner.with(|| {
            let store = SimulatorStore::new(Today::new(2025, 1, 2), storage);
            f(&store)
        });
        owner.unset();
        result
    }

    #[test]
    fn a_blocked_plan_sends_nothing_and_logs_the_reasons() {
        let client = FakeClient::never_touches_the_wire();
        with_store(None, |store| {
            block_on(store.execute(&client, options()));
            let run = store.run.get_untracked();
            assert_eq!(run.status, SimRunStatus::Failed);
            assert_eq!(run.run_id, None);
            // The plan's issues are the log, one `Blocked: …` each.
            assert_eq!(run.logs[0], "Blocked: Choose a symbol to simulate.");
            assert_eq!(client.compile_calls.get(), 0);
            assert_eq!(client.submit_calls.get(), 0);
            assert_eq!(client.journal_calls.get(), 0);
            // The wipe-before-run already happened even on the blocked path.
            assert_eq!(store.journal.get_untracked().status, JournalStatus::Idle);
            assert!(store.last_plan().is_some());
        });
    }

    #[test]
    fn a_failed_compile_fails_the_run_without_submitting() {
        let client = FakeClient::failing_compile();
        with_store(None, |store| {
            store.set_controls(valid_controls());
            block_on(store.execute(&client, options()));
            let run = store.run.get_untracked();
            assert_eq!(run.status, SimRunStatus::Failed);
            assert_eq!(
                run.error.as_deref(),
                Some("Compile failed: network: broken")
            );
            assert_eq!(client.compile_calls.get(), 1);
            assert_eq!(client.submit_calls.get(), 0);
            assert_eq!(client.journal_calls.get(), 0);
        });
    }

    #[test]
    fn a_completed_run_loads_the_journal_and_persists_the_selection() {
        let client = FakeClient::successful_run();
        let storage = std::sync::Arc::new(FakeStorage::default());
        with_store(Some(storage.clone() as Arc<dyn LayoutStorage>), |store| {
            store.set_controls(valid_controls());
            block_on(store.execute(&client, options()));
            let run = store.run.get_untracked();
            assert_eq!(run.status, SimRunStatus::Completed);
            assert_eq!(run.run_id.as_deref(), Some("r1"));
            assert_eq!(client.compile_calls.get(), 1);
            assert_eq!(client.submit_calls.get(), 1);
            assert_eq!(client.poll_calls.get(), 1);
            assert_eq!(client.journal_calls.get(), 1);

            let journal = store.journal.get_untracked();
            assert_eq!(journal.status, JournalStatus::Ready);
            assert_eq!(journal.run_id.as_deref(), Some("r1"));
            assert_eq!(journal.trades.len(), 2);

            // The plan was recorded and its strategy is *not* secretly the
            // compiled id (the id only rides the submitted request, as fe did).
            assert_eq!(
                store.last_plan().unwrap().request.strategy,
                Some("SMA Cross".to_string())
            );

            // The trips memo pairs the journal fills.
            assert_eq!(store.trips.get_untracked().len(), 1);

            // ...and the selection persisted (fe's persistSelection tail).
            let restored = load_persisted(storage.as_ref()).expect("selection persisted");
            assert_eq!(restored.strategy_name, "SMA Cross");
        });
    }

    #[test]
    fn a_stuck_run_folds_to_failed_at_the_poll_cap_with_no_journal() {
        let client = FakeClient::stuck_run();
        with_store(None, |store| {
            store.set_controls(valid_controls());
            let mut run_options = options();
            run_options.max_polls = 2;
            block_on(store.execute(&client, run_options));
            let run = store.run.get_untracked();
            assert_eq!(run.status, SimRunStatus::Failed);
            assert_eq!(
                run.error.as_deref(),
                Some("run r1 did not finish within 2 polls")
            );
            assert_eq!(client.poll_calls.get(), 2);
            assert_eq!(store.journal.get_untracked().status, JournalStatus::Idle);
        });
    }

    #[test]
    fn a_superseded_run_discards_a_late_journal_answer() {
        let mut client = FakeClient::successful_run();
        with_store(None, |store| {
            // The "newer fold" lands first: bump the generation while the
            // journal answer is in flight, exactly as an execute after this
            // one (or a reset) would.
            client.before_journal = Some({
                let run_generation = store.run_generation;
                Box::new(move || run_generation.update(|value| *value += 1))
            });
            store.set_controls(valid_controls());
            block_on(store.execute(&client, options()));
            let journal = store.journal.get_untracked();
            // The load was attempted but its answer was refused — the state
            // keeps the wiped-loading shape instead of stale fills.
            assert_eq!(client.journal_calls.get(), 1);
            assert_eq!(journal.status, JournalStatus::Loading);
            assert_eq!(journal.run_id.as_deref(), Some("r1"));
            assert!(journal.trades.is_empty());
        });
    }

    #[test]
    fn reset_run_clears_the_lifecycle_and_bumps_the_generation() {
        with_store(None, |store| {
            store.set_controls(valid_controls());
            block_on(store.execute(&FakeClient::stuck_run(), options()));
            let before = store.run_generation.get_untracked();
            store.reset_run();
            assert_eq!(store.run.get_untracked().status, SimRunStatus::Idle);
            assert_eq!(store.journal.get_untracked().status, JournalStatus::Idle);
            assert!(store.last_plan().is_none());
            assert!(store.run_generation.get_untracked() > before);
        });
    }

    #[test]
    fn set_controls_patches_the_panel_and_the_plan_refollows() {
        with_store(None, |store| {
            // Default controls have no symbol → the plan is blocked.
            assert!(!store.plan.get_untracked().issues.is_empty());
            store.set_controls(valid_controls());
            let plan = store.plan.get_untracked();
            assert!(plan.issues.is_empty());
            assert_eq!(plan.request.universe, Some("TCS.NSE".to_string()));
            assert_eq!(store.controls.get_untracked().symbol, "TCS");
            // The graph's template timeframe drove the request's bar spec.
            assert_eq!(plan.request.bar_spec, Some("5m".to_string()));
        });
    }

    #[test]
    fn select_adopts_a_graph_and_persists_the_selection() {
        let storage = FakeStorage::default();
        with_store(Some(Arc::new(storage)), |store| {
            let (graph, _) = builtin_template();
            {
                let mut named = graph.clone();
                named.name = "Renamed".to_string();
                named.driving_interval = 15;
                store.select(named, "Renamed", SelectionOrigin::Designer);
            }
            assert_eq!(store.strategy_name.get_untracked(), "Renamed");
            assert_eq!(store.origin.get_untracked(), SelectionOrigin::Designer);
            // The driving timeframe pre-filled the panel; the window stayed.
            assert_eq!(store.controls.get_untracked().timeframe, "15m");
            // The selection landed in storage.
            assert!(
                store
                    .storage
                    .as_ref()
                    .and_then(
                        |storage| storage.get_item(crate::sim::persistence::SIMULATOR_STORAGE_KEY)
                    )
                    .is_some()
            );
        });
    }

    #[test]
    fn hydrate_restores_a_persisted_selection() {
        let storage = FakeStorage::default();
        let controls = {
            let mut controls = default_controls();
            controls.symbol = "RELIANCE".to_string();
            controls.timeframe = "1d".to_string();
            controls
        };
        persist_selection(
            &SimulatorPersisted {
                strategy_name: "My TCS".to_string(),
                source: SelectionOrigin::Designer,
                controls: controls.clone(),
            },
            &storage,
        );
        with_store(Some(Arc::new(storage)), |store| {
            store.hydrate();
            assert_eq!(store.strategy_name.get_untracked(), "My TCS");
            assert_eq!(store.origin.get_untracked(), SelectionOrigin::Designer);
            assert_eq!(store.controls.get_untracked().symbol, "RELIANCE");
            // The graph itself is the template until the studio lands (P8):
            // the panel's universe field names the template's own instrument.
            assert_eq!(
                store.strategy.get_untracked().universe_explicit,
                vec![("TCS".to_string(), "NSE".to_string())]
            );
        });
    }

    // -- Replay wiring (P7 store-integration slice) --------------------------

    /// fe `engine.test.ts` `mockBars` — the same generator the controller
    /// tests use, so the store tests run over a real shape too.
    fn chart_bars(count: usize) -> Vec<ChartBar> {
        let mut bars = Vec::with_capacity(count);
        let mut price = 100.0;
        let base_time = 1_700_000_000;
        for i in 0..count {
            let delta = (i as f64 / 3.0).sin() * 2.0 + if i.is_multiple_of(2) { 1.0 } else { -0.5 };
            let open = price;
            let close = price + delta;
            price = close;
            bars.push(ChartBar {
                time: base_time + (i as i64) * 300,
                open,
                high: open.max(close) + 1.0,
                low: open.min(close) - 1.0,
                close,
                volume: 1000.0,
            });
        }
        bars
    }

    #[test]
    fn the_dock_tabs_spell_fes_dock_names() {
        assert_eq!(SimTab::Signals.as_str(), "signals");
        assert_eq!(SimTab::Portfolio.as_str(), "portfolio");
    }

    #[test]
    fn init_replay_seeds_the_engine_from_the_chart_dataset_and_the_controls() {
        with_store(None, |store| {
            let bars = chart_bars(5);
            store.set_controls(ControlsPatch {
                capital: Some(50_000.0),
                commission: Some(12.0),
                fill_model: Some("l2".to_string()),
                ..Default::default()
            });
            store.init_replay(&bars, "TCS.NSE");

            let state = store.replay_state();
            assert_eq!(state.total_bars, 5);
            assert_eq!(store.replay.get_value().total_bars(), 5);
            assert_eq!(store.replay.get_value().status(), ReplayStatus::Idle);
            // fe's init: symbol/exchange split, the panel's capital and the
            // fill settings land on the controller; the account stays the
            // sandbox default.
            assert_eq!(state.selected_account_id, "acc-sandbox-primary");
            assert_eq!(state.initial_capital, 50_000.0);
            assert_eq!(state.commission, 12.0);
            assert_eq!(state.fill_model, "l2");

            // The generation moved; the visible slice stays `None` (idle
            // shows the full loaded dataset).
            assert!(store.replay_generation.get_untracked() >= 1);
            assert!(store.replay_visible.get_untracked().is_none());

            // The config fields that make the engine *trade* this
            // instrument are not in the state snapshot (fe kept the field
            // set verbatim); read the controller's config accessor.
            let controller = store.replay.get_value();
            assert_eq!(controller.config().symbol, "TCS");
            assert_eq!(controller.config().exchange, "NSE");

            // Empty bars never initialise (fe guarded `bars.length`).
            store.init_replay(&[], "TCS.NSE");
            assert_eq!(store.replay_state().total_bars, 5);
        });
    }

    #[test]
    fn speed_survives_a_reinit_and_restarts_play_cadence() {
        with_store(None, |store| {
            let bars = chart_bars(4);
            store.init_replay(&bars, "TCS.NSE");
            store.replay_set_speed(25.0);
            assert_eq!(store.replay_state().speed, 25.0);

            // fe's init leaves the speed alone; a dataset change re-inits.
            store.init_replay(&bars, "TCS.NSE");
            assert_eq!(store.replay_state().speed, 25.0);

            // Non-playing speed changes do not touch the driver token.
            let before = store.replay_driver.get_untracked();
            store.replay_set_speed(50.0);
            assert_eq!(store.replay_driver.get_untracked(), before);
            assert_eq!(store.replay_state().speed, 50.0);
        });
    }

    #[test]
    fn play_step_and_pause_move_the_visible_slice_and_the_books() {
        with_store(None, |store| {
            let bars = chart_bars(4);
            store.init_replay(&bars, "TCS.NSE");

            // fe's play() emitted the first tick immediately.
            store.replay_play();
            assert_eq!(store.replay_state().status, ReplayStatus::Playing);
            let visible = store.replay_visible.get_untracked().expect("slice");
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].time, bars[0].time);
            assert_eq!(visible[0].close, bars[0].open);

            // Step one tick then pause (the controller pauses on step).
            store.replay_step();
            assert_eq!(store.replay_state().status, ReplayStatus::Paused);
            store.replay_pause();
            let visible = store.replay_visible.get_untracked().expect("slice");
            assert_eq!(visible.len(), 1);

            // Reset returns to idle; the slice falls back to nothing.
            store.replay_reset();
            assert_eq!(store.replay_state().status, ReplayStatus::Idle);
            let state = store.replay_state();
            assert_eq!(state.current_bar_index, 0);
            assert_eq!(state.realized_pnl, 0.0);
            assert_eq!(state.unrealized_pnl, 0.0);
            assert!(store.replay_visible.get_untracked().is_none());
        });
    }

    #[test]
    fn an_instant_run_completes_and_the_books_surface_the_trades() {
        with_store(None, |store| {
            let bars = chart_bars(30);
            store.init_replay(&bars, "TCS.NSE");
            store.replay_instant();

            let state = store.replay_state();
            assert_eq!(state.status, ReplayStatus::Completed);
            assert_eq!(state.current_bar_index, bars.len() - 1);
            // The engine's books are the portfolio surface: either trades
            // closed or a position still holds — the store never invents
            // account rows for the panel either way.
            assert!(
                !state.round_trips.is_empty() || state.active_position.is_some(),
                "an oscillating dataset must produce visible book activity"
            );
            // Equity reconciles: initial + realized + unrealized.
            let expected = state.initial_capital + state.realized_pnl + state.unrealized_pnl;
            assert!((state.equity - expected).abs() < 1e-9);
            // The visible slice reads the full dataset at completion.
            assert_eq!(
                store.replay_visible.get_untracked().map(|v| v.len()),
                Some(bars.len())
            );

            // TheEngine-state memo is stable across re-reads.
            assert_eq!(store.replay_state(), state);
        });
    }
}
