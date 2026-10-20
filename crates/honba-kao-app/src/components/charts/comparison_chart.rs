//! Two-series comparison chart. Mirrors
//! `web/src/components/charts/ComparisonChart.tsx`.

use leptos::prelude::*;

use crate::components::charts::{ChartSeries, OpenAlgoChart};
use honba_kao_ui_core::{HfPanel, HfPanelHeader};
use crate::data::demo_data::demo_ohlc_bars;

#[component]
pub fn ComparisonChart() -> impl IntoView {
    let portfolio = demo_ohlc_bars(1_820_000.0, 60, 86_400);
    let benchmark = demo_ohlc_bars(1_780_000.0, 60, 86_400);

    view! {
        <HfPanel>
            <HfPanelHeader title="Portfolio vs NIFTY">
                <span class="legend">
                    <span class="legend__dot legend__dot--a" />"portfolio"
                    <span class="legend__dot legend__dot--b" />"benchmark"
                </span>
            </HfPanelHeader>

            <div class="hfui-panel__content chart">
                <OpenAlgoChart
                    series=vec![
                        ChartSeries::line(portfolio, "--primary-8"),
                        ChartSeries::line(benchmark, "--shade-5"),
                    ]
                    class="openalgo-chart"
                    label="Portfolio versus benchmark"
                />
            </div>
        </HfPanel>
    }
}
