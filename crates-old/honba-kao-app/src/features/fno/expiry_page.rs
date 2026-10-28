//! Expiry Analysis (Phase 3.4a extension, `/fno/expiry`) — structural
//! placeholder. Real expiry-day analytics are tracked separately in
//! `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn ExpiryPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Expiry Analysis"
            subtitle="Expiry-day positioning, max pain and rollover tracking."
            sections=vec![
                ("Max Pain", "Theoretical max-pain strike per expiry versus spot."),
                ("Rollover Tracker", "Open-interest rollover from the near to the next expiry."),
            ]
        />
    }
}
