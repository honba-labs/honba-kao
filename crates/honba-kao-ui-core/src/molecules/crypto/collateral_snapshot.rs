//! `HfCollateralSnapshot` — summary card of account health: total collateral
//! value, borrow capacity, and utilization. Ported from
//! `secured-finance-app` molecule `CollateralSnapshot`.

use leptos::prelude::*;

/// Summary card of account collateral health and borrow capacity.
#[component]
pub fn HfCollateralSnapshot(
    #[prop(into)] total_collateral_value: f64,
    #[prop(into)] borrow_capacity: f64,
    #[prop(into)] margin_utilization_pct: f64,
) -> impl IntoView {
    let headroom = (borrow_capacity - total_collateral_value * margin_utilization_pct / 100.0).max(0.0);
    let zone_class = if margin_utilization_pct < 60.0 {
        "text-emerald-400"
    } else if margin_utilization_pct < 80.0 {
        "text-amber-400"
    } else {
        "text-rose-400"
    };

    view! {
        <div class="hf-collateral-snapshot rounded-lg border border-border-soft bg-surface-base p-3 text-xs shadow-sm">
            <span class="text-[10px] font-semibold text-text-muted uppercase tracking-wide block mb-2">"Collateral Snapshot"</span>
            <div class="hf-metrics grid grid-cols-2 gap-2 mb-2">
                <div class="hf-metric p-1.5 rounded bg-surface-subtle border border-border-soft/50">
                    <span class="text-[10px] text-text-muted block">"Total Collateral"</span>
                    <span class="font-mono text-text-high">{format!("₹{:.2}", total_collateral_value)}</span>
                </div>
                <div class="hf-metric p-1.5 rounded bg-surface-subtle border border-border-soft/50">
                    <span class="text-[10px] text-text-muted block">"Borrow Capacity"</span>
                    <span class="font-mono text-text-high">{format!("₹{:.2}", borrow_capacity)}</span>
                </div>
            </div>
            <div class="hf-utilization flex items-center gap-2">
                <span class="text-[10px] text-text-muted">"Margin Utilization"</span>
                <span class=format!("font-mono text-text-high {}", zone_class)>
                    {format!("{:.1}%", margin_utilization_pct)}
                </span>
                <span class="ml-auto text-[10px] text-text-muted">{format!("Headroom: ₹{:.2}", headroom.max(0.0))}</span>
            </div>
        </div>
    }
}
