//! Simulator — the strategy runner / backtester.
//!
//! The Leptos port of honba-fe's `SimulatorView`: the 56px replay command
//! bar (strategy/symbol/timeframe/date-range pickers, the compact replay
//! transport, the live P&L + confluence chips, the Parameters toggle), the
//! chart arena with its collapsible parameters drawer, and the six-tab
//! institutional bottom dock (`overview | trades | portfolio | signals |
//! logs | report`) — fe's exact `dockTab` union (see
//! [`SimTab`](crate::features::simulator::stores::simulator_store::SimTab)).
//!
//! The page owns a page-scoped [`SimulatorStore`]; every component below
//! reads the same signals, so the command bar, the chart, the drawer and
//! the dock stay consistent through the execute lifecycle (blocked →
//! running → terminal → journal) and the replay lifecycle (idle → playing
//! → paused → completed). A debug-build fixture replays a recorded run so
//! the page renders honestly against real wire shapes without a server.

use std::sync::Arc;

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::shared::api::http::client;
use crate::features::simulator::components::bottom_dock::BottomDock;
use crate::features::simulator::components::command_bar::SimCommandBar;
use crate::features::simulator::components::params_drawer::ParamsDrawer;
use crate::features::simulator::components::simulator_chart::SimulatorChart;
#[cfg(debug_assertions)]
use crate::features::simulator::kernel::journal::{JournalState, JournalStatus};
use crate::features::simulator::kernel::persistence::LocalStorage;
#[cfg(debug_assertions)]
use crate::features::simulator::kernel::run::initial_run_state;
use crate::features::simulator::kernel::run::{RunAbort, SimRunStatus};
use crate::features::simulator::kernel::selection::universe_string;
#[cfg(target_arch = "wasm32")]
use crate::features::simulator::stores::controls_store::utc_today;
#[cfg(target_arch = "wasm32")]
use crate::features::simulator::stores::run_store::{BrowserClock, run_options};
use crate::features::simulator::stores::simulator_store::SimulatorStore;

/// The debug fixture's completed run payload (the client crate's
/// `backtest_response_completed.json` shapes, embedded verbatim — no
/// hand-invented metrics).
#[cfg(debug_assertions)]
const DEMO_RESPONSE: &str = r#"{
  "run_id": "2025-01-02-v1-0-0-b7a1c9de834f2a019b3c5d6e7f809a1b",
  "status": "completed",
  "metrics": {
    "trades": 14,
    "net_pnl": 45230.55,
    "sharpe": 1.87,
    "max_drawdown": 0.0621,
    "total_return": 0.4523
  },
  "assumptions": {
    "not_modelled": [
      "market impact",
      "wire-in latency",
      "stamp duty"
    ]
  }
}"#;

/// The debug fixture's journal fills (the client crate's
/// `trades_response.json`, embedded verbatim; pairing derives the trips).
#[cfg(debug_assertions)]
const DEMO_TRADES: &str = r#"{
  "trades": [
    {
      "order_id": "BK-1-1700002205000000000",
      "instrument_id": { "symbol": "TCS", "exchange": "NSE" },
      "side": "buy",
      "quantity": 10.0,
      "price": 22010.0,
      "costs": { "amount": 0, "currency": "INR" },
      "ts_event": { "iso": "2023-11-14T22:50:05.000000000Z", "unix_nanos": "1700002205000000000" },
      "ts_init": { "iso": "2023-11-14T22:50:05.000000000Z", "unix_nanos": "1700002205000000000" }
    },
    {
      "order_id": "BK-1-1700006705000000000",
      "instrument_id": { "symbol": "TCS", "exchange": "NSE" },
      "side": "sell",
      "quantity": 10.0,
      "price": 22060.0,
      "costs": { "amount": 0, "currency": "INR" },
      "ts_event": { "iso": "2023-11-15T00:05:05.000000000Z", "unix_nanos": "1700006705000000000" },
      "ts_init": { "iso": "2023-11-15T00:05:05.000000000Z", "unix_nanos": "1700006705000000000" }
    },
    {
      "order_id": "BK-1-1700009705000000000",
      "instrument_id": { "symbol": "TCS", "exchange": "NSE" },
      "side": "buy",
      "quantity": 5.0,
      "price": 21980.0,
      "costs": { "amount": 0, "currency": "INR" },
      "ts_event": { "iso": "2023-11-15T00:55:05.000000000Z", "unix_nanos": "1700009705000000000" },
      "ts_init": { "iso": "2023-11-15T00:55:05.000000000Z", "unix_nanos": "1700009705000000000" }
    }
  ]
}"#;

/// Debug-build only: replays the recorded run into the store so the results
/// surface renders its real wire shapes. A fixture, not a fabrication — the
/// payload carries nothing beyond the recorded response, the journal pairing
/// still derives every number, and the button is labelled `Demo fixture`.
#[cfg(debug_assertions)]
fn load_demo_run(store: &SimulatorStore) {
    use crate::features::simulator::kernel::run::SimRunState;
    use honba_kao_client::wire::{BacktestResponse, TradesResponse};

    let response: BacktestResponse =
        serde_json::from_str(DEMO_RESPONSE).expect("demo run payload parses");
    let trades: TradesResponse = serde_json::from_str(DEMO_TRADES).expect("demo journal parses");

    // A demo run must not resurrect over an in-flight execute: bump the
    // generation exactly like `RunStore::reset` does, so any late poll/journal
    // fold of a previous attempt is rejected by the same guard.
    store
        .run
        .run_generation
        .update(|generation| *generation += 1);
    let run_id = response.run_id.clone();

    let mut run_state: SimRunState = initial_run_state();
    run_state.status = SimRunStatus::Completed;
    run_state.run_id = Some(run_id.clone());
    run_state.response = Some(response);
    run_state.logs = vec![
        "Demo: recorded run replayed (debug fixture)".to_string(),
        format!("Run {run_id} → completed"),
    ];
    store.run.state.set(run_state);

    store.run.journal.set(JournalState {
        status: JournalStatus::Ready,
        run_id: Some(run_id),
        trades: trades.trades,
        error: None,
    });
    // The demo never submitted a request: `last_plan` deliberately stays
    // `None`, so the payload tab reports "no request has been submitted"
    // rather than a fabricated one.
}

#[component]
pub fn SimulatorPage() -> impl IntoView {
    // The simulator is page-scoped: the store lives here so the memos root
    // into this component's reactive owner. `utc_today` is the browser's
    // clock (wasm); the native build never runs this page, so any compiled
    // day stands in there.
    #[cfg(target_arch = "wasm32")]
    let today = utc_today();
    #[cfg(not(target_arch = "wasm32"))]
    let today = crate::features::simulator::kernel::controls::Today::new(1970, 1, 1);
    let store = SimulatorStore::new(today, Some(Arc::new(LocalStorage)));
    store.controls.hydrate();

    // -- Chart inputs -------------------------------------------------------
    let controls = store.controls.panel;
    let strategy = store.controls.strategy;
    let bar_spec_line = Memo::new(move |_| controls.with(|controls| controls.timeframe.clone()));
    // The chart's bar window rides the plan (the panel's resolved range), not
    // the run: fe's chart is useful before anything was simulated.
    let bars_window = Memo::new(move |_| {
        store.controls.plan.with(|plan| {
            Some(honba_api_wasm::analytics::RunWindow {
                start: plan.range.0.clone(),
                end: plan.range.1.clone(),
            })
        })
    });
    let instrument_id = Memo::new(move |_| {
        let graph = strategy.get();
        controls.with(|controls| universe_string(&graph, controls))
    });
    let chart_run = store.run.state;
    let chart_status = Memo::new(move |_| {
        chart_run.with(|state| {
            if state.status == SimRunStatus::Running {
                "Running…".to_string()
            } else {
                match state.run_id.as_deref() {
                    Some(run_id) => format!("Run {run_id}"),
                    None => "Ready for simulation".to_string(),
                }
            }
        })
    });

    // -- Replay wiring ------------------------------------------------------
    // The chart publishes its loaded dataset into the store; the engine is
    // (re-)initialised whenever the fe initKey tuple moves: the bars fold,
    // the instrument, the timeframe and the execution controls. Every
    // driver loop dies on the bump, so a re-init never leaves a live timer
    // ticking a retired controller.
    let engine_bars = store.replay.engine_bars;
    let replay_visible_signal = store.replay.visible;
    let init_bars = store.replay.engine_bars;
    let init_instrument = instrument_id;
    let init_controls = store.controls.panel;
    let init_strategy = store.controls.strategy;
    let init_store = store.clone();
    Effect::new(move |_| {
        let bars = init_bars.get();
        let instrument = init_instrument.get();
        // fe's re-init effect listed `strategy` in its dependency array
        // (its `lastInitKey` string dedup was React strict-mode double-run
        // insurance; this effect runs once per actual tracked change, so a
        // same-shape graph edit re-initialises the books — the dep
        // array's own behaviour).
        let _graph_tick = init_strategy.get();
        let _controls_tick = init_controls.get();
        if bars.bars.is_empty() {
            return;
        }
        init_store.init_replay(&bars.bars, &instrument);
    });
    // The driver loop holds no owner handle; retiring it on unmount keeps a
    // background frame loop from outliving the page (fe cleared its
    // interval timer in the same position).
    on_cleanup(move || store.replay.driver.update(|token| *token += 1));

    // The page's one engine-state memo — the command bar's replay cluster,
    // live P&L chip and confluence pill, the overview KPI cards and the
    // dock's tab counts all track the same replay generation (fe's
    // `engineState` from `useSimulationEngine`), so a bar tick repaints
    // every surface from one source instead of re-deriving the controller's
    // state per reader.
    let replay = store.replay;
    let engine_state = Memo::new(move |_| {
        replay.generation.track();
        replay.state()
    });

    #[cfg(target_arch = "wasm32")]
    let bars_seam: Option<crate::features::simulator::components::equity_card::BarsSeamRef> =
        Some(Arc::new(client()));
    #[cfg(not(target_arch = "wasm32"))]
    let bars_seam: Option<crate::features::simulator::components::equity_card::BarsSeamRef> = None;

    // -- Execute wiring -------------------------------------------------------
    // The lifecycle runs on the browser's task queue and dies with the page;
    // `on_cleanup` aborts an in-flight loop exactly like fe's
    // `AbortController` teardown did (the fold lands in `failed`).
    let abort_flag = RwSignal::new(None::<RunAbort>);
    let exec_store = store.clone();
    let on_run = Callback::new(move |()| {
        if let Some(previous) = abort_flag.get_untracked() {
            previous.abort();
        }
        let abort = RunAbort::new();
        abort_flag.set(Some(abort.clone()));

        let store = exec_store.clone();
        // The execute future is not `Send` (it borrows this thread's
        // signals), so it runs on the current-task queue only; the native
        // build never renders this page.
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::task::spawn_local;
            spawn_local(async move {
                let api = client();
                let clock = BrowserClock;
                let options = run_options(&clock, abort);
                store.execute(&api, options).await;
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (&store, &abort);
    });
    let reset_store = store.clone();
    let on_clear = Callback::new(move |()| reset_store.run.reset());

    #[cfg(debug_assertions)]
    let on_demo = {
        let demo_store = RwSignal::new(store.clone());
        Some(Callback::new(move |()| demo_store.with_untracked(load_demo_run)))
    };
    #[cfg(not(debug_assertions))]
    let on_demo: Option<Callback<()>> = None;

    on_cleanup(move || {
        if let Some(abort) = abort_flag.get_untracked() {
            abort.abort();
        }
    });

    let is_params_open = store.is_params_open;
    let store_cmd = store.clone();
    let store_params = store.clone();
    let store_dock = store.clone();

    view! {
        <div class="simulator-view-root">
            // -- Command bar: selection + replay transport + run lifecycle --
            <SimCommandBar
                store=store_cmd
                engine_state=engine_state
                instrument_id=instrument_id
            />

            // -- Main workspace: chart arena + collapsible params drawer ----
            <section class="sim-workspace-body">
                <div class="sim-chart-arena">
                    <div class="sim-chart-wrapper">
                        <SimulatorChart
                            instrument_id=instrument_id
                            bars_window=bars_window
                            bar_spec=bar_spec_line
                            status_note=chart_status
                            strategy=strategy
                            bars=bars_seam.clone()
                            on_bars=Callback::new(move |state| engine_bars.set(state))
                            replay_bars=replay_visible_signal
                        />
                    </div>
                </div>

                <Show when=move || is_params_open.get()>
                    <ParamsDrawer
                        store=store_params.clone()
                        on_run=on_run
                        on_clear=on_clear
                        on_demo=on_demo
                    />
                </Show>
            </section>

            // -- Bottom dock: fe's exact six tabs ---------------------------
            <BottomDock store=store_dock engine_state=engine_state bars=bars_seam.clone() />
        </div>
    }
}
