//! `HfPriceYieldItem` — two-tier display coupling cash price with derived APR, YTM,
//! or option IV. Ported from `secured-finance-app` atom `PriceYieldItem`.

use leptos::prelude::*;

#[component]
pub fn HfPriceYieldItem(
    #[prop(into)] price: String,
    #[prop(into)] yield_label: String,
    #[prop(into, optional)] is_positive: Option<bool>,
) -> impl IntoView {
    let positive = is_positive.unwrap_or(true);
    let color = if positive {
        "text-emerald-400"
    } else {
        "text-rose-400"
    };
    view! {
        <div class="hf-price-yield-item flex flex-col gap-0.5">
            <span class="text-xs font-mono text-text-high">{price}</span>
            <span class=format!("text-[10px] font-mono {}", color)>{yield_label}</span>
        </div>
    }
}
