//! Strategy Builder — AI-native, Python-coded, sandboxed execution — scaffold.

use leptos::prelude::*;

use honba_kao_ui_core::{HfPageHeader, HfPanel, HfPanelHeader, HfPlaceholder};

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    view! {
        <div class="console">
            <HfPageHeader
                title="Strategy Builder"
                subtitle="AI-native, Python-coded strategies · sandboxed CSR execution"
            />

            <HfPanel>
                <HfPanelHeader title="Studio" />
                <div class="hfui-panel__content">
                    <HfPlaceholder
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
