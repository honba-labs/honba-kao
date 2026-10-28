//! `HfAssetAllocationBar` — segmented bar showing Equity, Debt, Cash, Gold weights.

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    pub equity: f64,
    pub debt: f64,
    pub cash: f64,
    pub gold: f64,
}

#[component]
pub fn HfAssetAllocationBar(
    alloc: Signal<Allocation>,
) -> impl IntoView {
    let total = Signal::derive(move || {
        let a = alloc.get();
        a.equity + a.debt + a.cash + a.gold
    });
    view! {
        <div class="hf-asset-allocation w-full h-2 rounded-full bg-surface-subtle overflow-hidden border border-border-soft">
            {move || {
                let t = total.get();
                let seg = |v: f64, _color: &'static str| {
                    if t > 0.0 { (v / t * 100.0).min(100.0) } else { 0.0 }
                };
                view! {
                    <div class="hf-segments flex h-full">
                        <div class="hf-segment bg-emerald-500" style=format!("width: {:.1}%;", seg(alloc.get().equity, "equity")) title="Equity" />
                        <div class="hf-segment bg-blue-500" style=format!("width: {:.1}%;", seg(alloc.get().debt, "debt")) title="Debt" />
                        <div class="hf-segment bg-amber-500" style=format!("width: {:.1}%;", seg(alloc.get().cash, "cash")) title="Cash" />
                        <div class="hf-segment bg-rose-500" style=format!("width: {:.1}%;", seg(alloc.get().gold, "gold")) title="Gold" />
                    </div>
                }
            }}
        </div>
    }
}
