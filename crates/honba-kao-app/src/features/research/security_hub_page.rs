use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn SecurityHubPage() -> impl IntoView {
    view! {
        <HfPage title="Security Research Hub" subtitle="Single-ticker comprehensive institutional dossier">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Fundamental metrics, balance sheet strength, management guidance, and consensus target prices."
            </div>
        </HfPage>
    }
}
