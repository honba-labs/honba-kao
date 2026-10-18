//! Equity curve. Mirrors `web/src/components/charts/EquityChart.tsx`.

use leptos::prelude::*;

use crate::components::charts::scale_points;
use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::data::demo_data::demo_price_series;

#[component]
pub fn EquityChart() -> impl IntoView {
    let series = demo_price_series(1_820_000.0, 60);
    let points = scale_points(&series, 100.0, 40.0);

    view! {
        <HfPanel>
            <HfPanelHeader title="Equity curve">
                <span class="muted small">"60 sessions"</span>
            </HfPanelHeader>

            <div class="hfui-panel__content chart">
                <svg
                    class="spark"
                    viewBox="0 0 100 40"
                    preserveAspectRatio="none"
                    role="img"
                    aria-label="Equity curve"
                >
                    <polyline
                        points=points
                        fill="none"
                        stroke="var(--primary-8)"
                        stroke-width="0.8"
                    />
                </svg>
            </div>
        </HfPanel>
    }
}
