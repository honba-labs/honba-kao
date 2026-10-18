//! Trading planning page (`/frontier`). Mirrors
//! `web/src/pages/TradingPlanningPage.tsx`.

use leptos::prelude::*;

use crate::components::charts::{ComparisonChart, EquityChart};
use crate::components::planning::TradingPlanning;
use crate::components::ui::PageHeader;

#[component]
pub fn TradingPlanningPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Trading planning"
                subtitle="Staged entries, per-leg triggers and the risk envelope"
            />

            <TradingPlanning />

            <div class="grid-2">
                <EquityChart />
                <ComparisonChart />
            </div>
        </div>
    }
}
