//! The simulator store (HBD-2026-DES-005 P4).
//!
//! The Leptos port of honba-fe `features/simulator/store/simulator-store.ts`.
//! Owns the active strategy graph, the control panel, the run lifecycle and
//! the replay engine: selection → plan → (blocked?) → compile → submit/poll →
//! journal. The pure logic lives in `kernel/*`; the stores are the thin signal
//! binding the pages and components read.
//!
//! The state lives in three slices that never touch one another's signals:
//! [`ControlsStore`] (selection, control panel, persistence, request plan),
//! [`RunStore`] (remote run lifecycle, journal, paired trips) and
//! [`ReplayStore`] (replay engine, playback clock, visible candle window).
//! The actions that span slices — executing the plan, (re-)initialising the
//! replay from the panel — and the analytics bundle derived across them live
//! here and pass state between slices explicitly.
//!
//! fe's zustand store kept every field in one bag; per the design doc's perf
//! note ("don't replicate the monolith store read") the state is a set of
//! **granular signals**, wrapped with `Memo`s for the plan, the paired round
//! trips and the analytics bundle — the chart, ribbon and journal re-render
//! independently.
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

use std::sync::Arc;

use honba_api_wasm::analytics::{
    AnalyticsInput, EquitySeries, JournalRow, MonthlyHeatmap, RibbonMetric, RoundTripStats,
    RunMetrics, RunWindow, derive_ribbon_metrics, equity_series, journal_rows, monthly_heatmap,
    round_trip_stats,
};
use honba_kao_client::wire::BacktestMetrics;
use leptos::prelude::*;

use super::controls_store::ControlsStore;
use super::replay_store::ReplayStore;
use super::run_store::RunStore;
use crate::features::simulator::kernel::chart_data::ChartBar;
use crate::features::simulator::kernel::controls::{SimulationRequestPlan, Today};
use crate::features::simulator::kernel::journal::JournalClient;
use crate::features::simulator::kernel::manifest::StrategyCompiler;
use crate::features::simulator::kernel::persistence::LayoutStorage;
use crate::features::simulator::kernel::run::{RunClient, RunOptions};

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
// The dock tabs
// ---------------------------------------------------------------------------

/// The bottom dock's tab strip — fe's exact `dockTab` union in
/// `SimulatorView.tsx` (`'overview' | 'trades' | 'portfolio' | 'signals' |
/// 'logs' | 'report'`). The heatmap/validation/payload tabs this build
/// briefly carried as separate dock tabs are folded into `Overview`
/// (equity + heatmap, like fe's live overview) and `Report` (the
/// `ReportPanel` — status, ribbon, validation badges, the raw payload —
/// the Leptos port of fe's `results-panel.tsx`), so the strip now spells
/// fe's six tabs one-for-one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SimTab {
    /// Metrics, header, actions.
    #[default]
    Overview,
    /// The trade journal table (fe's `trades` dock).
    Trades,
    /// The sandbox-local book — engine trips + open position (fe's
    /// `portfolio` dock).
    Portfolio,
    /// The replay engine's confluence detail (fe's `signals` dock).
    Signals,
    /// Run log + SSE event tail (fe's `logs` dock).
    Logs,
    /// The server backtest report: status, ribbon, validation, payload
    /// (fe's `report` dock / `results-panel.tsx`).
    Report,
}

impl SimTab {
    /// Every tab, in fe's strip order.
    pub const ALL: [SimTab; 6] = [
        SimTab::Overview,
        SimTab::Trades,
        SimTab::Portfolio,
        SimTab::Signals,
        SimTab::Logs,
        SimTab::Report,
    ];

    /// fe's spelling, shown in the strip / read by tests and persistence.
    pub const fn as_str(self) -> &'static str {
        match self {
            SimTab::Overview => "overview",
            SimTab::Trades => "trades",
            SimTab::Portfolio => "portfolio",
            SimTab::Signals => "signals",
            SimTab::Logs => "logs",
            SimTab::Report => "report",
        }
    }

    /// The tab button's label text (fe's JSX label next to the icon).
    pub const fn label(self) -> &'static str {
        match self {
            SimTab::Overview => "Overview & Performance",
            SimTab::Trades => "Trade Journal",
            SimTab::Portfolio => "Sandboxed Portfolio",
            SimTab::Signals => "Signal Confluence",
            SimTab::Logs => "Execution Logs",
            SimTab::Report => "Server Backtest Report",
        }
    }
}

// ---------------------------------------------------------------------------
// The coordinator
// ---------------------------------------------------------------------------

/// The simulator store: one handle over the three slices, the dock tab and
/// the analytics bundle derived across them.
///
/// Construct inside a reactive owner (a component body or a test's runtime);
/// the memos are rooted at construction. Cloning shares the same signals —
/// the pages hand the store down and the execute future keeps a clone.
#[derive(Clone)]
pub struct SimulatorStore {
    /// Selection, control panel, persistence and the request plan.
    pub controls: ControlsStore,
    /// The remote run lifecycle, journal and paired trips.
    pub run: RunStore,
    /// The replay engine and its reactive surface.
    pub replay: ReplayStore,
    /// The bottom dock's active tab (`dockTab`).
    pub tab: RwSignal<SimTab>,
    /// The analytics bundle for the results surface.
    pub analytics_bundle: Memo<SimAnalytics>,
    /// The collapsible parameters drawer's open state (fe's `isParamsOpen`).
    pub is_params_open: RwSignal<bool>,
}

impl SimulatorStore {
    /// A fresh store: template selection, default controls, idle run +
    /// journal, idle replay.
    pub fn new(today: Today, storage: Option<Arc<dyn LayoutStorage>>) -> Self {
        let controls = ControlsStore::new(today, storage);
        let run = RunStore::new();
        let replay = ReplayStore::new();
        let tab = RwSignal::new(SimTab::Overview);

        let analytics_bundle = {
            let run_state = run.state;
            let trips_memo = run.trips;
            let last_plan = run.last_plan;
            let panel = controls.panel;
            Memo::new(move |_| {
                let run = run_state.get();
                let trips = trips_memo.get();
                let last_plan = last_plan.get();
                let capital = panel.get().capital;
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
                let run_metrics = metrics.as_ref().map(|metrics| RunMetrics {
                    trades: metrics.trades,
                    net_pnl: metrics.net_pnl.to_string().parse::<f64>().unwrap_or(0.0),
                    sharpe: metrics.sharpe.to_string().parse::<f64>().unwrap_or(0.0),
                    max_drawdown: metrics
                        .max_drawdown
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    total_return: metrics
                        .total_return
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                });
                let input = AnalyticsInput {
                    metrics: run_metrics.as_ref(),
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
            controls,
            run,
            replay,
            tab,
            analytics_bundle,
            is_params_open: RwSignal::new(false),
        }
    }

    // -- Execute (controls → run) --------------------------------------------

    /// Submits the current plan for the current graph (fe `execute`) and, once
    /// the attempt reached the backtest route, persists the selection (fe's
    /// `persistSelection` tail). See [`RunStore::execute`] for the lifecycle.
    pub async fn execute<C>(&self, client: &C, options: RunOptions<'_>)
    where
        C: StrategyCompiler + RunClient + JournalClient,
    {
        let plan = self.controls.plan.get_untracked();
        let graph = self.controls.strategy.get_untracked();
        if self.run.execute(client, options, plan, &graph).await {
            self.controls.persist();
        }
    }

    // -- Replay init (controls → replay) -------------------------------------

    /// Re-initialises the replay over the chart's published `bars` with the
    /// current panel and graph (fe's `initKey` re-init).
    pub fn init_replay(&self, bars: &[ChartBar], instrument_id: &str) {
        let controls = self.controls.panel.get_untracked();
        let graph = self.controls.strategy.get_untracked();
        self.replay.init(bars, instrument_id, &controls, &graph);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::simulator::kernel::journal::JournalStatus;
    use crate::features::simulator::kernel::persistence::load_persisted;
    use crate::features::simulator::kernel::run::{ClientFut, Clock, SimRunStatus};
    use crate::features::simulator::kernel::testing::block_on;
    use crate::features::simulator::stores::controls_store::ControlsPatch;
    use honba_kao_client::ClientError;
    use honba_kao_client::wire::{
        BacktestRequest, BacktestResponse, CompiledStrategy, Exchange, InstrumentId, RunStatus,
        Trade, TradesResponse, WireOrderSide,
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
                "manifest": {
                    "api_version": "1.0.0",
                    "name": "SMA Cross",
                    "source_hash": "sha256:abc",
                    "universe": { "explicit": [ { "symbol": "TCS", "exchange": "NSE" } ] },
                    "subscriptions": {
                        "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ]
                    },
                    "driving_timeframe": { "interval": 5, "aggregation": "minute" },
                    "warmup_bars": 2
                },
                "subscriptions": {
                    "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ],
                    "bars": [ { "interval": 5, "aggregation": "minute" } ],
                    "quotes": true,
                    "trades": true
                },
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
        use rust_decimal::Decimal;
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Completed,
            metrics: Some(honba_kao_client::wire::BacktestMetrics {
                trades: 2,
                net_pnl: Decimal::from(500),
                sharpe: Decimal::new(12, 1),
                max_drawdown: Decimal::new(5, 2),
                total_return: Decimal::new(5, 2),
            }),
            assumptions: None,
            validation: None,
            error: None,
        }
    }

    fn fill(minute: i64) -> Trade {
        let ts = chrono::DateTime::from_timestamp(1_704_153_600 + minute * 60, 0)
            .expect("valid timestamp");
        Trade::new(
            "f",
            "o",
            honba_core::uuid::Uuid::nil(),
            InstrumentId::new(Exchange::Nse, "TCS"),
            WireOrderSide::Buy,
            honba_core::Price::new(rust_decimal::Decimal::from(100)),
            honba_core::Quantity::from_u64(10),
            "algo",
            ts,
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
            let run = store.run.state.get_untracked();
            assert_eq!(run.status, SimRunStatus::Failed);
            assert_eq!(run.run_id, None);
            // The plan's issues are the log, one `Blocked: …` each.
            assert_eq!(run.logs[0], "Blocked: Choose a symbol to simulate.");
            assert_eq!(client.compile_calls.get(), 0);
            assert_eq!(client.submit_calls.get(), 0);
            assert_eq!(client.journal_calls.get(), 0);
            // The wipe-before-run already happened even on the blocked path.
            assert_eq!(
                store.run.journal.get_untracked().status,
                JournalStatus::Idle
            );
            assert!(store.run.last_plan().is_some());
        });
    }

    #[test]
    fn a_failed_compile_fails_the_run_without_submitting() {
        let client = FakeClient::failing_compile();
        with_store(None, |store| {
            store.controls.patch(valid_controls());
            block_on(store.execute(&client, options()));
            let run = store.run.state.get_untracked();
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
            store.controls.patch(valid_controls());
            block_on(store.execute(&client, options()));
            let run = store.run.state.get_untracked();
            assert_eq!(run.status, SimRunStatus::Completed);
            assert_eq!(run.run_id.as_deref(), Some("r1"));
            assert_eq!(client.compile_calls.get(), 1);
            assert_eq!(client.submit_calls.get(), 1);
            assert_eq!(client.poll_calls.get(), 1);
            assert_eq!(client.journal_calls.get(), 1);

            let journal = store.run.journal.get_untracked();
            assert_eq!(journal.status, JournalStatus::Ready);
            assert_eq!(journal.run_id.as_deref(), Some("r1"));
            assert_eq!(journal.trades.len(), 2);

            // The plan was recorded and its strategy is *not* secretly the
            // compiled id (the id only rides the submitted request, as fe did).
            assert_eq!(
                store.run.last_plan().unwrap().request.strategy,
                Some("SMA Cross".to_string())
            );

            // The trips memo pairs the journal fills.
            assert_eq!(store.run.trips.get_untracked().len(), 1);

            // ...and the selection persisted (fe's persistSelection tail).
            let restored = load_persisted(storage.as_ref()).expect("selection persisted");
            assert_eq!(restored.strategy_name, "SMA Cross");
        });
    }

    #[test]
    fn a_stuck_run_folds_to_failed_at_the_poll_cap_with_no_journal() {
        let client = FakeClient::stuck_run();
        with_store(None, |store| {
            store.controls.patch(valid_controls());
            let mut run_options = options();
            run_options.max_polls = 2;
            block_on(store.execute(&client, run_options));
            let run = store.run.state.get_untracked();
            assert_eq!(run.status, SimRunStatus::Failed);
            assert_eq!(
                run.error.as_deref(),
                Some("run r1 did not finish within 2 polls")
            );
            assert_eq!(client.poll_calls.get(), 2);
            assert_eq!(
                store.run.journal.get_untracked().status,
                JournalStatus::Idle
            );
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
                let run_generation = store.run.run_generation;
                Box::new(move || run_generation.update(|value| *value += 1))
            });
            store.controls.patch(valid_controls());
            block_on(store.execute(&client, options()));
            let journal = store.run.journal.get_untracked();
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
            store.controls.patch(valid_controls());
            block_on(store.execute(&FakeClient::stuck_run(), options()));
            let before = store.run.run_generation.get_untracked();
            store.run.reset();
            assert_eq!(store.run.state.get_untracked().status, SimRunStatus::Idle);
            assert_eq!(
                store.run.journal.get_untracked().status,
                JournalStatus::Idle
            );
            assert!(store.run.last_plan().is_none());
            assert!(store.run.run_generation.get_untracked() > before);
        });
    }

    #[test]
    fn the_dock_tabs_spell_fes_dock_names() {
        assert_eq!(SimTab::Signals.as_str(), "signals");
        assert_eq!(SimTab::Portfolio.as_str(), "portfolio");
        assert_eq!(SimTab::Trades.as_str(), "trades");
        assert_eq!(SimTab::Report.as_str(), "report");
        assert_eq!(SimTab::ALL.len(), 6);
    }
}
