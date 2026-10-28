use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    view! {
        <HfPage title="Strategy Studio & Signal Engine" subtitle="Visual signal rule builder, condition chaining, and code exporter">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Rule-based event trigger composition, trailing stop definitions, and bracket order wiring."
            </div>
        </HfPage>
    }
}
