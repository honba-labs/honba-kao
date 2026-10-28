use super::components::ScreenerHeroDeck;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn ScreenerPage() -> impl IntoView {
    view! {
        <HfPage title="Institutional Stock Screener" subtitle="Vector predicate engine with AST query builder and multi-metric filtering">
            <ScreenerHeroDeck />
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Fast vector evaluation across 2,000+ symbols. AST filter conditions, technical triggers, and fundamental ratios."
            </div>
        </HfPage>
    }
}
