//! Workbench — charting, buy/sell and analysis — scaffold.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader, Placeholder};

#[component]
pub fn WorkbenchPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Workbench"
                subtitle="Charting, order tickets and per-symbol analysis"
            />

            <HfPanel>
                <HfPanelHeader title="Chart workspace" />
                <div class="hfui-panel__content">
                    <Placeholder
                        route="workbench"
                        note="The single-symbol cockpit: charts with drawing tools,
                        buy/sell order tickets and inline analysis. Hand the current
                        symbol to the Simulator or a strategy from here."
                    />
                </div>
            </HfPanel>
        </div>
    }
}
