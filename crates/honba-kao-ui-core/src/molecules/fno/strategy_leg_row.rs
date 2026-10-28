//! `HfStrategyLegRow` — leg builder item with buy/sell side, strike, expiry,
//! and lot multiplier.

use leptos::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq)]
pub struct StrategyLeg {
    pub side: String, // "BUY" / "SELL"
    pub strike: Decimal,
    pub expiry: String,
    pub lots: Decimal,
}

#[component]
pub fn HfStrategyLegRow(
    leg: Signal<StrategyLeg>,
    #[prop(into)] on_remove: Callback<()>,
) -> impl IntoView {
    let side_color = Signal::derive(move || {
        if leg.get().side == "BUY" { "text-emerald-400" } else { "text-rose-400" }
    });
    view! {
        <div class="hf-strategy-leg-row flex justify-between items-center px-3 py-1.5 rounded border border-border-soft bg-surface-base text-xs">
            <div class="hf-leg-side flex items-center gap-1">
                <span class=move || format!("{} font-bold", side_color.get())>{move || leg.get().side.clone()}</span>
                <span class="text-text-muted">" × "</span>
                <span class="font-mono">{move || format!("{}", leg.get().lots.round_dp(2))}</span>
            </div>
            <div class="hf-leg-strike text-center font-mono text-text-high">
                {move || format!("{}", leg.get().strike.round_dp(2))}
            </div>
            <span class="text-text-muted">{move || leg.get().expiry.clone()}</span>
            <button
                type="button"
                class="hf-leg-remove text-text-muted hover:text-text-high"
                on:click=move |_| on_remove.run(())
                aria-label="Remove leg"
            >
                <span class="hf-icon hf-icon-close" aria_hidden="true">"✕"</span>
            </button>
        </div>
    }
}
