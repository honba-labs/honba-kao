//! Tradebook — the journal of past analyses and runs — scaffold.

use leptos::prelude::*;

use honba_kao_ui_core::{HfPageHeader, HfPanel, HfPanelHeader, HfPlaceholder};

#[component]
pub fn TradebookPage() -> impl IntoView {
    view! {
        <div class="console">
            <HfPageHeader
                title="Tradebook"
                subtitle="Every past analysis — session, month, year-wise"
            />

            <HfPanel>
                <HfPanelHeader title="History" />
                <div class="hfui-panel__content">
                    <HfPlaceholder
                        route="tradebook"
                        note="Log of past sessions, months and years: analyses,
                        simulator runs, signals taken and outcomes, grouped by
                        session / month / year."
                    />
                </div>
            </HfPanel>
        </div>
    }
}
