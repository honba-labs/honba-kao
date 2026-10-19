//! Strategy Builder — AI-native, Python-coded, sandboxed execution — scaffold.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader, Placeholder};

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Strategy Builder"
                subtitle="AI-native, Python-coded strategies · sandboxed CSR execution"
            />

            <HfPanel>
                <HfPanelHeader title="Studio" />
                <div class="hfui-panel__content">
                    <Placeholder
                        route="strategy builder"
                        note="Python strategy code with AI assistance, executed in a
                        sandboxed CSR runtime. Strategies bind to a symbol or a
                        curated portfolio and emit buy/sell signals."
                    />
                </div>
            </HfPanel>
        </div>
    }
}
