use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn BondsPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
            "Nelson-Siegel sovereign yield curve interpolation, dirty/clean bond pricing, and modified duration telemetry."
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Yield Curve"</h3>
            <span class="text-xs text-text-muted">"G-Sec, SDL, and corporate tenors."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Bonds & Fixed Income Workstation"
            subtitle="Sovereign G-Secs, SDLs, corporate paper, and yield term structure"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
