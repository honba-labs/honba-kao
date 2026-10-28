use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn OiAnalysisPage() -> impl IntoView {
    view! {
        <HfPage title="Open Interest Buildup & Max Pain" subtitle="PCR ratios, unwinding indicators, and strike-wise OI shifts">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "PCR volume vs open interest, Max Pain strike trajectory, and institutional positioning."
            </div>
        </HfPage>
    }
}
