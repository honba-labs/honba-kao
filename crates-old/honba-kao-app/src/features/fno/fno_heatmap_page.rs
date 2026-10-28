//! Derivatives Heatmap (Phase 3.4a extension, `/fno/heatmap`) — structural
//! placeholder. Real strike/expiry OI-change heatmap is tracked separately
//! in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn FnoHeatmapPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Derivatives Heatmap"
            subtitle="Strike x expiry open-interest change heatmap."
            sections=vec![
                ("OI Change Grid", "Strike rows against expiry columns, coloured by OI build-up or unwind."),
                ("Hotspot Legend", "Call vs. put build-up colour key and ATM strike marker."),
            ]
        />
    }
}
