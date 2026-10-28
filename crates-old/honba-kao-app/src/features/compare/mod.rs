//! Charts domain (plan.md §2.1 "Charts") — the Compare page, which overlays
//! several symbols on one normalized chart. Structural placeholder; the
//! real multi-symbol overlay chart is tracked separately in `plan.md`'s
//! phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn ComparePage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Compare"
            subtitle="Normalized multi-symbol performance overlay."
            sections=vec![
                ("Symbol Basket", "Add up to a handful of symbols to compare on one normalized axis."),
                ("Overlay Chart", "Percentage-return overlay with a shared crosshair and legend."),
                ("Correlation Strip", "Pairwise correlation summary for the selected basket."),
            ]
        />
    }
}
