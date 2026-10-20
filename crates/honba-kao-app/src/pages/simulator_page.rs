//! Simulator — the strategy runner / backtester (port phase P5).
//!
//! The Leptos port of honba-fe's `SimulatorView` results surface: the run
//! toolbar (selection, live request plan, execute action), the empty chart
//! arena (P6 places the candle chart there; the realised **equity card**
//! with its labelled benchmark overlay already reads the journal from the
//! overview tab) and the bottom dock with the
//! `overview | journal | heatmap | validation | logs | payload` tabs —
//! fe's `signals`/`portfolio` docks follow with the replay engine (P7).
//!
//! The page owns a page-scoped [`SimulatorStore`]; every component below
//! reads the same signals, so the run header, ribbon, journal and heat map
//! stay consistent through the execute lifecycle (blocked → running →
//! terminal → journal). A debug-build fixture replays a recorded run so the
//! page renders honestly against real wire shapes without a server.

use std::sync::Arc;

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::api::http::client;
use crate::components::simulator::equity_card::EquityCard;
use crate::components::simulator::journal_table::JournalTable;
use crate::components::simulator::metrics_ribbon::MetricsRibbon;
use crate::components::simulator::monthly_heatmap::MonthlyHeatmap;
use crate::components::simulator::portfolio_panel::PortfolioPanel;
use crate::components::simulator::replay_controls::ReplayControls;
use crate::components::simulator::run_log::RunLogDrawer;
use crate::components::simulator::signals_panel::SignalsPanel;
use crate::components::simulator::simulator_chart::SimulatorChart;
use crate::components::simulator::validation_badges::ValidationBadges;
use honba_kao_ui_core::HfPageHeader;
#[cfg(debug_assertions)]
use crate::sim::journal::{JournalState, JournalStatus};
use crate::sim::persistence::LocalStorage;
#[cfg(debug_assertions)]
use crate::sim::run::initial_run_state;
use crate::sim::run::{RunAbort, SimRunStatus};
use crate::sim::selection::universe_string;
#[cfg(target_arch = "wasm32")]
use crate::stores::simulator::utc_today;
#[cfg(target_arch = "wasm32")]
use crate::stores::simulator::{BrowserClock, run_options};
use crate::stores::simulator::{SimTab, SimulatorStore};

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
    use crate::sim::run::SimRunState;
    use honba_kao_client::wire::{BacktestResponse, TradesResponse};

    let response: BacktestResponse =
        serde_json::from_str(DEMO_RESPONSE).expect("demo run payload parses");
    let trades: TradesResponse = serde_json::from_str(DEMO_TRADES).expect("demo journal parses");

    // A demo run must not resurrect over an in-flight execute: bump the
    // generation exactly like `reset_run` does, so any late poll/journal
    // fold of a previous attempt is rejected by the same guard.
    store.run_generation.update(|generation| *generation += 1);
    let run_id = response.run_id.clone();

    let mut run_state: SimRunState = initial_run_state();
    run_state.status = SimRunStatus::Completed;
    run_state.run_id = Some(run_id.clone());
    run_state.response = Some(response);
    run_state.logs = vec![
        "Demo: recorded run replayed (debug fixture)".to_string(),
        format!("Run {run_id} → completed"),
    ];
    store.run.set(run_state);

    store.journal.set(JournalState {
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
    let today = crate::sim::controls::Today::new(1970, 1, 1);
    let store = SimulatorStore::new(today, Some(Arc::new(LocalStorage)));
    store.hydrate();

    // -- Derived reads the components consume --------------------------------
    let analytics = store.analytics_bundle;
    let metrics = Memo::new(move |_| analytics.with(|bundle| bundle.ribbon.clone()));
    let heatmap = Memo::new(move |_| analytics.with(|bundle| bundle.heatmap.clone()));
    let wire_total_return = Memo::new(move |_| {
        analytics.with(|bundle| bundle.metrics.as_ref().map(|metrics| metrics.total_return))
    });
    let rows = Memo::new(move |_| analytics.with(|bundle| bundle.rows.clone()));
    let export_name = Memo::new(move |_| store.strategy_name.with(|name| name.clone()));

    // -- Equity card inputs (P6) -------------------------------------------
    let run_generation = store.run_generation;
    let controls = store.controls;
    let strategy = store.strategy;
    let equity_series = Memo::new(move |_| analytics.with(|bundle| bundle.equity_series.clone()));
    let run_window = Memo::new(move |_| analytics.with(|bundle| bundle.window.clone()));
    let benchmark_label = Memo::new(move |_| controls.with(|controls| controls.benchmark.clone()));
    let bar_spec_line = Memo::new(move |_| controls.with(|controls| controls.timeframe.clone()));
    // The chart's bar window rides the plan (the panel's resolved range), not
    // the run: fe's chart is useful before anything was simulated.
    let bars_window = Memo::new(move |_| {
        store.plan.with(|plan| {
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
    let chart_run = store.run;
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

    // -- Replay wiring (P7) ------------------------------------------------
    // The chart publishes its loaded dataset into the store; the engine is
    // (re-)initialised whenever the fe initKey tuple moves: the bars fold,
    // the instrument, the timeframe and the execution controls. Every
    // driver loop dies on the bump, so a re-init never leaves a live timer
    // ticking a retired controller.
    let engine_bars = store.engine_bars;
    let replay_visible_signal = store.replay_visible;
    let init_bars = store.engine_bars;
    let init_instrument = instrument_id;
    let init_controls = store.controls;
    let init_strategy = store.strategy;
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
    on_cleanup(move || store.replay_driver.update(|token| *token += 1));

    #[cfg(target_arch = "wasm32")]
    let bars_seam: Option<crate::components::simulator::equity_card::BarsSeamRef> =
        Some(Arc::new(client()));
    #[cfg(not(target_arch = "wasm32"))]
    let bars_seam: Option<crate::components::simulator::equity_card::BarsSeamRef> = None;

    // The validation slice off the run response; nothing is inferred when a
    // payload carried no block.
    let run = store.run;
    let validation = Memo::new(move |_| {
        run.with(|state| {
            state
                .response
                .as_ref()
                .and_then(|response| response.validation.clone())
        })
    });
    let not_modelled = Memo::new(move |_| {
        run.with(|state| {
            state
                .response
                .as_ref()
                .and_then(|response| response.assumptions.clone())
        })
        .and_then(|assumptions| assumptions.get("not_modelled").cloned())
        .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
        .unwrap_or_default()
    });

    // -- Toolbar readouts ------------------------------------------------
    let name = store.strategy_name;
    let controls = store.controls;
    let status_line = Memo::new(move |_| run.with(|state| state.status.as_str().to_string()));
    let running = Memo::new(move |_| run.with(|state| state.status == SimRunStatus::Running));
    let run_id_line =
        Memo::new(move |_| run.with(|state| state.run_id.clone().unwrap_or_default()));
    let error_line = Memo::new(move |_| run.with(|state| state.error.clone().unwrap_or_default()));
    let last_plan = store.last_plan;
    let plan_window = Memo::new(move |_| {
        last_plan.with(|plan| {
            plan.as_ref()
                .map(|plan| format!("{} → {}", plan.range.0, plan.range.1))
        })
    });
    let blocked = Memo::new(move |_| {
        last_plan.with(|plan| {
            plan.as_ref()
                .map(|plan| !plan.issues.is_empty())
                .unwrap_or(false)
        })
    });

    // -- Execute wiring ----------------------------------------------------
    // The lifecycle runs on the browser's task queue and dies with the page;
    // `on_cleanup` aborts an in-flight loop exactly like fe's
    // `AbortController` teardown did (the fold lands in `failed`). The Run
    // button starts the spawn; the closure stays Fn because only the spawn
    // future owns the store clone.
    let abort_flag = RwSignal::new(None::<RunAbort>);
    let exec_store = store.clone();
    let on_run = move || {
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
    };
    let reset_store = store.clone();
    #[cfg(debug_assertions)]
    let demo_store = RwSignal::new(store.clone());

    on_cleanup(move || {
        if let Some(abort) = abort_flag.get_untracked() {
            abort.abort();
        }
    });

    // Copyable handles for the view closures.
    let tab = store.tab;

    view! {
        <div class="console">
            <HfPageHeader
                title="Simulator"
                subtitle="Backtester · run the active strategy, journal its fills, read the results"
            />

            // -- Toolbar: selection + run lifecycle + execute ---------------
            <div class="sim-toolbar">
                <span class="sim-toolbar__title">{move || name.get()}</span>
                <span class="sim-toolbar__meta">
                    {move || controls.get().symbol.clone()}
                    " · "
                    {move || {
                        plan_window.get().unwrap_or_else(|| "window unresolved".to_string())
                    }}
                </span>
                <span class=move || {
                    format!("sim-status sim-status--{}", status_line.get())
                }>
                    {move || status_line.get()}
                </span>
                <Show when=move || !run_id_line.get().is_empty()>
                    <span class="sim-toolbar__meta">{move || run_id_line.get()}</span>
                </Show>
                <span class="sim-toolbar__actions">
                    <button
                        type="button"
                        class="sim-toolbar__btn"
                        disabled=move || running.get()
                        on:click=move |_| on_run()
                    >
                        "Run"
                    </button>
                    <button
                        type="button"
                        class="sim-toolbar__btn sim-toolbar__btn--ghost"
                        title="Clear the run and journal"
                        on:click=move |_| reset_store.reset_run()
                    >
                        "Clear"
                    </button>
                    <Show
                        when=move || cfg!(debug_assertions)
                        fallback=move || view! { <span /> }
                    >
                        <button
                            type="button"
                            class="sim-toolbar__btn sim-toolbar__btn--ghost"
                            title="Debug: replay a recorded completed run"
                            on:click=move |_| demo_store.with_untracked(load_demo_run)
                        >
                            "Demo fixture"
                        </button>                    </Show>
                </span>
            </div>
            <Show when=move || !error_line.get().is_empty()>
                <div class="sim-run-error">{move || error_line.get()}</div>
            </Show>
            <Show when=move || blocked.get()>
                <div class="sim-run-error">
                    "The panel is incomplete and the request is blocked — nothing was sent."
                </div>
            </Show>

            // -- Chart arena (P6): bar container; volume pane, indicator
            // overlays and markers follow the bridge work. The P7 replay
            // wiring hands the loaded dataset to the store (one fetch,
            // shared — fe read the chart's dataLayer bars) and lets the
            // engine's visible slice replace the candles while a replay
            // holds the view.
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

            // -- Replay transport (P7) -------------------------------------
            <ReplayControls store=store.clone() />

            // -- Bottom dock ------------------------------------------------
            <div class="sim-dock">
                <div class="sim-dock-tabs" role="tablist">
                    {[
                        SimTab::Overview,
                        SimTab::Journal,
                        SimTab::Signals,
                        SimTab::Portfolio,
                        SimTab::Heatmap,
                        SimTab::Validation,
                        SimTab::Logs,
                        SimTab::Payload,
                    ]
                        .iter()
                        .map(|side| {
                            let key = *side;
                            view! {
                                <button
                                    type="button"
                                    role="tab"
                                    class=move || {
                                        if tab.get() == key {
                                            "sim-dock-tab sim-dock-tab--active".to_string()
                                        } else {
                                            "sim-dock-tab".to_string()
                                        }
                                    }
                                    on:click=move |_| tab.set(key)
                                >
                                    {key.as_str()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>

                {move || match tab.get() {
                    SimTab::Overview => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <MetricsRibbon metrics=metrics />
                            <EquityCard
                                equity=equity_series
                                window=run_window
                                benchmark_label=benchmark_label
                                bar_spec=bar_spec_line
                                run_generation=run_generation
                                bars=bars_seam.clone()
                            />
                        </div>
                    }
                        .into_any(),
                    SimTab::Journal => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <JournalTable rows=rows export_name=export_name />
                        </div>
                    }
                        .into_any(),
                    SimTab::Signals => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <SignalsPanel store=store.clone() />
                        </div>
                    }
                        .into_any(),
                    SimTab::Portfolio => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <PortfolioPanel store=store.clone() />
                        </div>
                    }
                        .into_any(),
                    SimTab::Heatmap => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <MonthlyHeatmap heatmap=heatmap wire_total_return=wire_total_return />
                        </div>
                    }
                        .into_any(),
                    SimTab::Validation => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <ValidationBadges validation=validation not_modelled=not_modelled />
                        </div>
                    }
                        .into_any(),
                    SimTab::Logs => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <RunLogDrawer run=run />
                        </div>
                    }
                        .into_any(),
                    SimTab::Payload => view! {
                        <div class="sim-dock-body" role="tabpanel">
                            <PayloadPreview last_plan />
                        </div>
                    }
                        .into_any(),
                }}
            </div>
        </div>
    }
}

/// The raw request the last execute submitted — fe showed
/// `JSON.stringify(lastPlan?.request, null, 2)`; until an execute has run
/// this tab reports the absence rather than inventing a request.
#[component]
fn PayloadPreview(
    /// `store.last_plan` — the submitted plan, or `None` before any run.
    last_plan: RwSignal<Option<crate::sim::controls::SimulationRequestPlan>>,
) -> impl IntoView {
    view! {
        {move || {
            last_plan
                .get()
                .and_then(|plan| serde_json::to_string_pretty(&plan.request).ok())
                .map(|payload| {
                    view! { <pre class="sim-payload">{payload}</pre> }.into_any()
                })
                .unwrap_or_else(|| {
                    view! { <div class="sim-empty">"No request has been submitted yet."</div> }
                        .into_any()
                })
        }}
    }
}
