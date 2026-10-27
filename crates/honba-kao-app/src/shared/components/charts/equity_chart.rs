//! Equity curve. Mirrors `web/src/components/charts/EquityChart.tsx`.

use leptos::prelude::*;

use crate::shared::components::charts::{ChartSeries, OpenAlgoChart};
use crate::shared::data::demo_data::demo_ohlc_bars;
use honba_kao_ui_core::{HfPanel, HfPanelHeader};

#[component]
pub fn EquityChart() -> impl IntoView {
    let bars = demo_ohlc_bars(1_820_000.0, 60, 86_400);

    view! {
        <HfPanel>
            <HfPanelHeader title="Equity curve">
                <span class="muted small">"60 sessions"</span>
            </HfPanelHeader>

            <div class="hfui-panel__content chart">
                <OpenAlgoChart
                    series=vec![ChartSeries::area(bars)]
                    class="openalgo-chart"
                    label="Equity curve"
                />
            </div>
        </HfPanel>
    }
}
