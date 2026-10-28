//! Put-Call Ratio (Phase 3.4a extension, `/fno/pcr`) — structural
//! placeholder. Real PCR-by-expiry and PCR-history charts are tracked
//! separately in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn PcrPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Put-Call Ratio"
            subtitle="OI and volume-based PCR for the active underlying."
            sections=vec![
                ("PCR by Expiry", "Open-interest and volume PCR broken down per expiry."),
                ("PCR History", "Rolling PCR trend used as a contrarian sentiment gauge."),
            ]
        />
    }
}
