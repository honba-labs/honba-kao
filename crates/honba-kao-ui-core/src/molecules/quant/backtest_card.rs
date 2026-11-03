//! `HfBacktestMetricCard` — single backtest performance metric tile.

use leptos::prelude::*;

#[component]
pub fn HfBacktestMetricCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into, optional)] positive: Option<bool>,
) -> impl IntoView {
    let positive = positive.unwrap_or(true);
    let color = if positive { "text-emerald-400" } else { "text-rose-400" };
    view! {
        <div class="hf-backtest-card rounded border border-border-soft bg-surface-base p-3">
            <span class="text-[10px] text-text-muted uppercase tracking-wider block">{label}</span>
            <div class="hf-value font-mono text-sm">{value}</div>
            <span class=format!("text-[10px] {}", color)></span>
        </div>
    }
}
