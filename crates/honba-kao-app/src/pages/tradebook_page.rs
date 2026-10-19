//! Tradebook — the journal of past analyses and runs — scaffold.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader, Placeholder};

#[component]
pub fn TradebookPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Tradebook"
                subtitle="Every past analysis — session, month, year-wise"
            />

            <HfPanel>
                <HfPanelHeader title="History" />
                <div class="hfui-panel__content">
                    <Placeholder
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
