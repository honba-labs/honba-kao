//! The six-tab institutional bottom dock — the Leptos port of honba-fe
//! `SimulatorView.tsx`'s `<footer className="sim-bottom-dock">`.
//!
//! Tabs (fe order, icons, live counts): Overview & Performance, Trade
//! Journal (rows), Sandboxed Portfolio (engine trips), Signal Confluence
//! (engine trade events), Execution Logs (log lines), Server Backtest
//! Report (`Ready` once a run answered). Contextual dock actions sit on the
//! right of the strip, like fe's: `Reset Sandbox P&L` on the portfolio tab.
//!
//! Sources (unchanged from kao's previous dock): the overview's KPI cards
//! read the replay books like fe's; its equity card and monthly heatmap,
//! and the trade journal, read the server run's analytics bundle. The
//! journal's `Export CSV` stays in the journal's own toolbar because it
//! exports the filtered, sorted rows the table holds.

use honba_api_wasm::analytics::{MetricKind, format_metric_value};
use honba_kao_ui_core::icons::outline::{
    activity, bar_chart_2, briefcase, delete, file_spreadsheet, scroll_text, shield_check,
};
use honba_kao_ui_core::{HfButton, SvgIcon};
use leptos::prelude::*;

use crate::features::simulator::components::equity_card::{BarsSeamRef, EquityCard};
use crate::features::simulator::components::journal_table::JournalTable;
use crate::features::simulator::components::kpi_cards::KpiCards;
use crate::features::simulator::components::monthly_heatmap::MonthlyHeatmap;
use crate::features::simulator::components::portfolio_panel::PortfolioPanel;
use crate::features::simulator::components::report_panel::ReportPanel;
use crate::features::simulator::components::run_log::RunLogDrawer;
use crate::features::simulator::components::signals_panel::SignalsPanel;
use crate::features::simulator::kernel::engine::controller::SimulationEngineState;
use crate::features::simulator::stores::simulator_store::{SimTab, SimulatorStore};

/// The tab's lucide-style icon (fe's `BarChart2`, `FileSpreadsheet`, …).
fn tab_icon(tab: SimTab) -> &'static str {
    match tab {
        SimTab::Overview => bar_chart_2(),
        SimTab::Trades => file_spreadsheet(),
        SimTab::Portfolio => briefcase(),
        SimTab::Signals => activity(),
        SimTab::Logs => scroll_text(),
        SimTab::Report => shield_check(),
    }
}

#[component]
pub fn BottomDock(
    /// The simulator store.
    store: SimulatorStore,
    /// The page's engine-state memo.
    #[prop(into)]
    engine_state: Signal<SimulationEngineState>,
    /// The bar seam the equity card's benchmark overlay reads.
    bars: Option<BarsSeamRef>,
) -> impl IntoView {
    let tab = store.tab;
    let analytics = store.analytics_bundle;
    let run = store.run.state;
    let replay = store.replay;

    // -- Server-run reads ----------------------------------------------------
    let rows = Memo::new(move |_| analytics.with(|bundle| bundle.rows.clone()));
    let heatmap = Memo::new(move |_| analytics.with(|bundle| bundle.heatmap.clone()));
    let wire_total_return = Memo::new(move |_| {
        analytics.with(|bundle| {
            bundle.metrics.as_ref().map(|metrics| {
                metrics
                    .total_return
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            })
        })
    });
    let sharpe = Memo::new(move |_| {
        analytics.with(|bundle| {
            bundle
                .metrics
                .as_ref()
                .map(|metrics| metrics.sharpe.to_string().parse::<f64>().unwrap_or(0.0))
        })
    });
    let equity_series = Memo::new(move |_| analytics.with(|bundle| bundle.equity_series.clone()));
    let run_window = Memo::new(move |_| analytics.with(|bundle| bundle.window.clone()));
    let panel = store.controls.panel;
    let benchmark_label = Memo::new(move |_| panel.with(|panel| panel.benchmark.clone()));
    let bar_spec = Memo::new(move |_| panel.with(|panel| panel.timeframe.clone()));
    let initial_capital = Memo::new(move |_| panel.with(|panel| panel.capital));
    let export_name = store.controls.strategy_name;
    let run_generation = store.run.run_generation;

    // -- Replay reads ----------------------------------------------------------
    let history = Memo::new(move |_| engine_state.with(|state| state.confluence_history.clone()));

    // -- Tab counts (fe's `(n)` suffixes) --------------------------------------
    let count = move |key: SimTab| -> Option<usize> {
        match key {
            SimTab::Trades => Some(rows.with(Vec::len)),
            SimTab::Portfolio => Some(engine_state.with(|state| state.round_trips.len())),
            SimTab::Signals => Some(history.with(Vec::len)),
            SimTab::Logs => Some(history.with(Vec::len) + run.with(|state| state.logs.len())),
            SimTab::Overview | SimTab::Report => None,
        }
    };
    let report_ready = Memo::new(move |_| run.with(|state| state.response.is_some()));

    let tabs = SimTab::ALL
        .iter()
        .map(|key| {
            let key = *key;
            view! {
                <HfButton
                    unstyled=true
                    class="sim-dock-tab-btn"
                    role="tab"
                    active=Signal::derive(move || tab.get() == key)
                    on_click=Callback::new(move |_| tab.set(key))
                >
                    <SvgIcon path=tab_icon(key) size=13 />
                    <span>
                        {move || match count(key) {
                            Some(count) => format!("{} ({count})", key.label()),
                            None => key.label().to_string(),
                        }}
                    </span>
                    {(key == SimTab::Report).then(|| view! {
                        <Show when=move || report_ready.get()>
                            <span class="sim-badge-ready">"Ready"</span>
                        </Show>
                    })}
                </HfButton>
            }
        })
        .collect_view();

    view! {
        <footer class="sim-bottom-dock">
            <div class="sim-dock-nav">
                <div class="sim-dock-tabs" role="tablist">{tabs}</div>
                <div class="sim-dock-actions">
                    <Show when=move || tab.get() == SimTab::Portfolio>
                        <HfButton
                            unstyled=true
                            class="sim-ghost-btn color-bearish"
                            title="Clear all sandboxed positions and reset P&L"
                            on_click=Callback::new(move |_| replay.reset())
                        >
                            <SvgIcon path=delete() size=12 />
                            <span>"Reset Sandbox P&L"</span>
                        </HfButton>
                    </Show>
                </div>
            </div>

            <div class="sim-dock-body" role="tabpanel">
                {move || match tab.get() {
                    SimTab::Overview => view! {
                        <div class="sim-overview-pane">
                            <KpiCards engine_state=engine_state sharpe=sharpe />
                            <div class="sim-split-grid">
                                <div class="sim-chart-card">
                                    <div class="sim-card-header">
                                        <div>
                                            <h4 class="sim-card-title">"Cumulative Strategy Equity Curve"</h4>
                                            <p class="sim-card-subtitle">
                                                {move || format!(
                                                    "Server-run realised equity vs. initial base (₹{})",
                                                    format_metric_value(Some(initial_capital.get()), MetricKind::Currency),
                                                )}
                                            </p>
                                        </div>
                                        {move || wire_total_return.get().map(|total| {
                                            let class = if total >= 0.0 {
                                                "sim-pill sim-pill--bullish"
                                            } else {
                                                "sim-pill sim-pill--bearish"
                                            };
                                            view! {
                                                <span class=class>
                                                    {format!(
                                                        "{} Total Return",
                                                        format_metric_value(
                                                            Some(total),
                                                            MetricKind::Percent,
                                                        ),
                                                    )}
                                                </span>
                                            }
                                        })}
                                    </div>
                                    <EquityCard
                                        equity=equity_series
                                        window=run_window
                                        benchmark_label=benchmark_label
                                        bar_spec=bar_spec
                                        run_generation=run_generation
                                        bars=bars.clone()
                                    />
                                </div>
                                <div class="sim-heatmap-card">
                                    <div class="sim-card-header">
                                        <div>
                                            <h4 class="sim-card-title">"Monthly Return Heatmap"</h4>
                                            <p class="sim-card-subtitle">
                                                "Consistency & monthly distribution of returns (%)"
                                            </p>
                                        </div>
                                    </div>
                                    <MonthlyHeatmap heatmap=heatmap wire_total_return=wire_total_return />
                                </div>
                            </div>
                        </div>
                    }
                        .into_any(),
                    SimTab::Trades => view! {
                        <JournalTable rows=rows export_name=export_name />
                    }
                        .into_any(),
                    SimTab::Portfolio => view! {
                        <PortfolioPanel store=store.clone() />
                    }
                        .into_any(),
                    SimTab::Signals => view! {
                        <div class="sim-signals-pane">
                            <div class="sim-signals-header">
                                <span class="sim-signals-title">
                                    "Live Multi-Indicator Confluence Evaluation"
                                </span>
                                <span class="sim-signals-stamp">
                                    {move || engine_state.with(|state| {
                                        let time = state
                                            .current_tick
                                            .as_ref()
                                            .map(|tick| tick.time_str.clone())
                                            .filter(|time| !time.is_empty())
                                            .unwrap_or_else(|| "—".to_string());
                                        format!("Evaluated at Bar {} · {time}", state.current_bar_index)
                                    })}
                                </span>
                            </div>
                            <SignalsPanel store=store.clone() />
                        </div>
                    }
                        .into_any(),
                    SimTab::Logs => view! {
                        <RunLogDrawer run=run />
                    }
                        .into_any(),
                    SimTab::Report => view! {
                        <ReportPanel store=store.clone() />
                    }
                        .into_any(),
                }}
            </div>
        </footer>
    }
}
