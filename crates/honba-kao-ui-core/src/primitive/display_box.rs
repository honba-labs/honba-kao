//! `HfOrderDisplayBox` — read-only tabular key-value display for execution prices,
//! yields, notionals (monospace tabular numerals).

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderDisplayItem {
    pub label: String,
    pub value: String,
}

/// Read-only tabular key-value display for execution prices, yields, notionals.
#[component]
pub fn HfOrderDisplayBox(items: Vec<OrderDisplayItem>) -> impl IntoView {
    view! {
        <div class="hf-order-display-box flex flex-col gap-1">
            {items.into_iter().map(|item| {
                view! {
                    <div class="hf-display-row flex justify-between text-xs font-mono text-text-high">
                        <span class="text-text-muted">{item.label}</span>
                        <span class="text-text-high">{item.value}</span>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
