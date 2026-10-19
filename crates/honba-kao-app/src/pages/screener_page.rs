//! Screener / market view — scaffold.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader, Placeholder};

#[component]
pub fn ScreenerPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Screener"
                subtitle="Market view — scan, filter and rank the universe"
            />

            <HfPanel>
                <HfPanelHeader title="Screen builder" />
                <div class="hfui-panel__content">
                    <Placeholder
                        route="market view + screener"
                        note="Scans and watchlists will live here: filter presets,
                        ranking columns and saved screens feeding the Workbench and
                        the Strategy Builder's curated portfolios."
                    />
                </div>
            </HfPanel>
        </div>
    }
}
