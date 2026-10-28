use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn MarketsPage() -> impl IntoView {
    view! {
        <HfPage title="Macro & Sector Heatmaps" subtitle="Breadth indicators, sector rotation, and global macro cross-currents">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Sector performance matrices, advance-decline ratios, and market breadth indicators."
            </div>
        </HfPage>
    }
}
