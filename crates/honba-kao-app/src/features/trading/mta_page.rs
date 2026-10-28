use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn MtaPage() -> impl IntoView {
    view! {
        <HfPage title="Multi-Timeframe Analysis (MTA)" subtitle="Confluence matrix across 1m, 5m, 15m, 1h, and Daily horizons">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Multi-timeframe trend alignment, VWAP bands, and volume profiles."
            </div>
        </HfPage>
    }
}
