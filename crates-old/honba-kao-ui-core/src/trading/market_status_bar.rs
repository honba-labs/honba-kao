//! MarketStatusBar — feed / OMS / risk connection status pills.

use leptos::prelude::*;

use crate::trading::pl_number::{HfStatus, HfStatusCircle};

/// Row of status pills for the market data feed, order management system and
/// risk gate. Each status is reactive.
#[component]
pub fn MarketStatusBar(
    #[prop(into)] feed: Signal<HfStatus>,
    #[prop(into)] oms: Signal<HfStatus>,
    #[prop(into)] risk: Signal<HfStatus>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("hf-market-status-bar {class}").trim().to_string() role="status">
            <StatusPill label="Feed" status=feed />
            <StatusPill label="OMS" status=oms />
            <StatusPill label="Risk" status=risk />
        </div>
    }
}

#[component]
fn StatusPill(label: &'static str, status: Signal<HfStatus>) -> impl IntoView {
    view! {
        <span class="hf-market-status-bar__pill">
            {move || view! { <HfStatusCircle status=status.get() /> }}
            <span class="hf-market-status-bar__label">{label}</span>
        </span>
    }
}
