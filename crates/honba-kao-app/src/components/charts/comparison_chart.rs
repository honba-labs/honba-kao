//! Two-series comparison chart. Mirrors
//! `web/src/components/charts/ComparisonChart.tsx`.

use leptos::prelude::*;

use crate::components::charts::scale_points;
use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::data::demo_data::demo_price_series;

#[component]
pub fn ComparisonChart() -> impl IntoView {
    let portfolio = demo_price_series(1_820_000.0, 60);
    let benchmark = demo_price_series(1_780_000.0, 60);
    let a = scale_points(&portfolio, 100.0, 40.0);
    let b = scale_points(&benchmark, 100.0, 40.0);

    view! {
        <HfPanel>
            <HfPanelHeader title="Portfolio vs NIFTY">
                <span class="legend">
                    <span class="legend__dot legend__dot--a" />"portfolio"
                    <span class="legend__dot legend__dot--b" />"benchmark"
                </span>
            </HfPanelHeader>

            <div class="hfui-panel__content chart">
                <svg
                    class="spark"
                    viewBox="0 0 100 40"
                    preserveAspectRatio="none"
                    role="img"
                    aria-label="Portfolio versus benchmark"
                >
                    <polyline points=a fill="none" stroke="var(--primary-8)" stroke-width="0.8" />
                    <polyline points=b fill="none" stroke="var(--shade-5)" stroke-width="0.8" stroke-dasharray="1.5 1" />
                </svg>
            </div>
        </HfPanel>
    }
}
