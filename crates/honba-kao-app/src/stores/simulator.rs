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

use std::rc::Rc;

use leptos::prelude::*;
use honba_kao_client::wire::BacktestMetrics;

use crate::sim::analytics::{
    derive_ribbon_metrics, equity_series, journal_rows, monthly_heatmap, round_trip_stats,
    AnalyticsInput, EquitySeries, JournalRow, MonthlyHeatmap, RoundTripStats, RunWindow,
    RibbonMetric,
};
use crate::sim::controls::{
    default_controls, plan_request, PlanOptions, PeriodPreset, SimulationRequestPlan,
    SimulatorControls, Today,
};
use crate::sim::journal::{
    apply_journal_loading, initial_journal_state, load_journal, JournalClient, JournalState,
};
use crate::sim::manifest::{compile_strategy, StrategyCompiler};
use crate::sim::pairing::{pair_trades, RoundTrip};
use crate::sim::persistence::{
    load_persisted, persist_selection, LayoutStorage, SimulatorPersisted,
};
use crate::sim::run::{
    initial_run_state, run_backtest, RunClient, RunOptions, SimRunState, SimRunStatus,
};
#[cfg(any(target_arch = "wasm32", test))]
use crate::sim::run::Clock;
use crate::sim::selection::{
    builtin_template, controls_for_graph, universe_string, SelectionOrigin, StrategyGraphShape,
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
            use wasm_bindgen::closure::Closure as WasmClosure;
            use wasm_bindgen::JsCast as _;

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
                    let _ = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
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
                .unwrap_or_default()
                .into();
        })
    }
}

// ---------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------

/// The simulator store: granular signals + derived memos, methods mirror fe's
/// `SimulatorState`.
///
/// Construct inside a reactive owner (a component body or a test's runtime);
/// the memos are rooted at construction.
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
    /// Best-effort handoff storage (fe: `localStorage`); `None` in tests.
    pub storage: Option<Rc<dyn LayoutStorage>>,

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
}

impl SimulatorStore {
    /// A fresh store: template selection, default controls, idle run +
    /// journal. fe's `initialState()` ran `resolveSelection({})`, whose
    /// template fallback is all this build resolves without the library.
    pub fn new(today: Today, storage: Option<Rc<dyn LayoutStorage>>) -> Self {
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

        let trips_memo = {
            Memo::new(move |_| pair_trades(&journal.get().trades).trips)
        };

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
            storage,
            plan: plan_memo,
            trips: trips_memo,
            analytics_bundle: analytics_memo,
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
            run.logs = plan.issues.iter().map(|issue| format!("Blocked: {issue}")).collect();
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

    /// The storage double (fe's localStorage stubs).
    #[derive(Default)]
    struct FakeStorage(RefCell<String>);

    impl LayoutStorage for FakeStorage {
        fn get_item(&self, _key: &str) -> Option<String> {
            let raw = self.0.borrow();
            (!raw.is_empty()).then(|| raw.clone())
        }

        fn set_item(&self, _key: &str, value: &str) {
            *self.0.borrow_mut() = value.to_string();
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
                journal: Ok(TradesResponse { trades: vec![fill(0), fill(10)] }),
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
    fn with_store<T>(storage: Option<Rc<dyn LayoutStorage>>, f: impl FnOnce(&SimulatorStore) -> T) -> T {
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
            assert_eq!(run.error.as_deref(), Some("Compile failed: network: broken"));
            assert_eq!(client.compile_calls.get(), 1);
            assert_eq!(client.submit_calls.get(), 0);
            assert_eq!(client.journal_calls.get(), 0);
        });
    }

    #[test]
    fn a_completed_run_loads_the_journal_and_persists_the_selection() {
        let client = FakeClient::successful_run();
        let storage = std::rc::Rc::new(FakeStorage::default());
        with_store(Some(storage.clone() as Rc<dyn LayoutStorage>), |store| {
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
            assert_eq!(store.last_plan().unwrap().request.strategy, Some("SMA Cross".to_string()));

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
        with_store(Some(Rc::new(storage)), |store| {
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
            assert!(store
                .storage
                .as_ref()
                .and_then(|storage| storage.get_item(crate::sim::persistence::SIMULATOR_STORAGE_KEY))
                .is_some());
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
        with_store(Some(Rc::new(storage)), |store| {
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
}
