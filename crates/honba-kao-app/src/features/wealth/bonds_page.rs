use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn BondsPage() -> impl IntoView {
    view! {
        <HfPage title="Bonds & Fixed Income Workstation" subtitle="Sovereign G-Secs, SDLs, corporate paper, and yield term structure">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Nelson-Siegel sovereign yield curve interpolation, dirty/clean bond pricing, and modified duration telemetry."
            </div>
        </HfPage>
    }
}
