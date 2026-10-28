//! `HfSipCalculatorCard` — compounding projection tool with monthly installment
//! slider. Projection math may use `honba-kao-api-wasm`; UI molecule here.

use leptos::prelude::*;
use crate::primitive::HfNumberInput;

#[component]
pub fn HfSipCalculatorCard(
    #[prop(into)] monthly: RwSignal<f64>,
    #[prop(into)] years: RwSignal<f64>,
    #[prop(into)] annual_return: RwSignal<f64>, // %
) -> impl IntoView {
    let projected = Signal::derive(move || {
        let m = monthly.get();
        let y = years.get();
        let r = annual_return.get() / 100.0 / 12.0;
        let n = (y * 12.0) as u32;
        if r > 0.0 {
            m * ((1.0 + r).powi(n as i32) - 1.0) / r
        } else {
            m * n as f64
        }
    });
    let invested = Signal::derive(move || monthly.get() * years.get() * 12.0);
    view! {
        <div class="hf-sip-calculator rounded-lg border border-border-soft bg-surface-base p-3 shadow-sm">
            <span class="text-xs font-semibold text-text-high uppercase tracking-wide mb-2 block">"SIP Calculator"</span>
            <div class="hf-sip-inputs grid grid-cols-2 gap-3 mb-3">
                <div>
                    <span class="text-[10px] text-text-muted block">"Monthly (₹)"</span>
                    <HfNumberInput value=monthly placeholder="0" />
                </div>
                <div>
                    <span class="text-[10px] text-text-muted block">"Years"</span>
                    <HfNumberInput value=years placeholder="0" min=0.0 step=1.0 />
                </div>
                <div>
                    <span class="text-[10px] text-text-muted block">"Return % p.a."</span>
                    <HfNumberInput value=annual_return placeholder="0" min=0.0 step=0.1 />
                </div>
                <div class="col-span-2">
                    <span class="text-[10px] text-text-muted block">"Projected Value (₹)"</span>
                    <div class="font-mono text-text-high text-sm">{move || format!("{:.2}", projected.get())}</div>
                </div>
            </div>
            <div class="hf-sip-split flex justify-between text-[10px] text-text-muted mt-1">
                <span>{move || format!("Invested: {:.2}", invested.get())}</span>
                <span class="text-emerald-400">{move || format!("Gains: {:.2}", projected.get() - invested.get())}</span>
            </div>
        </div>
    }
}
