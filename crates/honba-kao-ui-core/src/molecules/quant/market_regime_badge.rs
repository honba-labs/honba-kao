//! `HfMarketRegimeBadge` — market regime indicator (bull / bear / range).

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    Bull,
    Bear,
    Range,
}

#[component]
pub fn HfMarketRegimeBadge(
    regime: Regime,
) -> impl IntoView {
    let (label, class) = match regime {
        Regime::Bull => ("Bullish", "text-emerald-400 bg-emerald-600/20 border-emerald-500/30"),
        Regime::Bear => ("Bearish", "text-rose-400 bg-rose-600/20 border-rose-500/30"),
        Regime::Range => ("Range-bound", "text-cyan-400 bg-cyan-600/20 border-cyan-500/30"),
    };
    view! {
        <span class=format!("hf-market-regime inline-flex items-center gap-1 rounded border px-2 py-0.5 text-[10px] font-bold {}", class)>
            {label}
        </span>
    }
}
