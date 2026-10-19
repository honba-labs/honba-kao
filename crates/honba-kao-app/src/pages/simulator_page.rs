//! Simulator — the strategy runner / backtester — scaffold.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader, Placeholder};

#[component]
pub fn SimulatorPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Simulator"
                subtitle="Backtester · multichart · animated bars · signal generation"
            />

            <HfPanel>
                <HfPanelHeader title="Run a strategy" />
                <div class="hfui-panel__content">
                    <Placeholder
                        route="simulator"
                        note="The strategy runner: portfolio chosen from the strategy
                        or the current Workbench symbol, multichart layout, animated
                        bar replay and buy/sell signal generation."
                    />
                </div>
            </HfPanel>
        </div>
    }
}
