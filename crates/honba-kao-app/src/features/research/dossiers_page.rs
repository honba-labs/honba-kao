use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn DossiersPage() -> impl IntoView {
    view! {
        <HfPage title="Institutional Research Reports" subtitle="Synthesized equity intelligence and analyst research coverage">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Deep-dive initiation notes, quarterly earnings previews, and channel check findings."
            </div>
        </HfPage>
    }
}
