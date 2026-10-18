//! Strategy studio page. Mirrors `web/src/pages/StrategyStudioPage.tsx`.

use leptos::prelude::*;

use crate::components::charts::AdvancedChart;
use crate::components::strategy::{GridRiskPanel, StrategyStudio};
use crate::components::ui::PageHeader;

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader title="Strategy studio" subtitle="Library, parameters and live charts" />

            <div class="grid-2">
                <AdvancedChart />
                <GridRiskPanel />
            </div>

            <StrategyStudio />
        </div>
    }
}
