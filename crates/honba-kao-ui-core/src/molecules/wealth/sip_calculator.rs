//! `HfSipCalculatorCard` — compounding projection tool with monthly installment
//! slider. Projection math may use `honba-kao-api-wasm`; UI molecule here.
//! All money values use `rust_decimal::Decimal` (Zero Floats Policy).

use crate::{Decimal, HfNumberInput};
use num_traits::ToPrimitive;
use rust_decimal_macros::dec;
use leptos::prelude::*;

#[component]
pub fn HfSipCalculatorCard(
    #[prop(into)] monthly: RwSignal<Decimal>,
    #[prop(into)] years: RwSignal<Decimal>,
    #[prop(into)] annual_return: RwSignal<Decimal>, // %
) -> impl IntoView {
    let projected = Signal::derive(move || {
        let m = monthly.get();
        let y = years.get();
        let r = annual_return.get() / Decimal::from(100) / Decimal::from(12);
        let n = (y * Decimal::from(12)).to_i32().unwrap_or(0) as u32;
        if r > Decimal::ZERO {
            let mut factor = Decimal::ONE;
            let one_plus_r = Decimal::ONE + r;
            for _ in 0..n {
                factor = factor * one_plus_r;
            }
            m * (factor - Decimal::ONE) / r
        } else {
            m * Decimal::from(n)
        }
    });
    let invested = Signal::derive(move || monthly.get() * years.get() * Decimal::from(12));

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
                    <HfNumberInput value=years placeholder="0" min=Decimal::ZERO step=dec!(1) />
                </div>
                <div>
                    <span class="text-[10px] text-text-muted block">"Return % p.a."</span>
                    <HfNumberInput value=annual_return placeholder="0" min=Decimal::ZERO step=dec!(0.1) />
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
