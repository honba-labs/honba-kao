use crate::molecules::fno::{HfOptionChainRow, OptionLegQuote};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct OptionStrikePair {
    pub strike: f64,
    pub call: OptionLegQuote,
    pub put: OptionLegQuote,
}

#[component]
pub fn HfOptionChain(
    #[prop(into)] spot_price: Signal<f64>,
    #[prop(into)] pairs: Signal<Vec<OptionStrikePair>>,
    #[prop(into)] on_select_leg: Callback<(f64, bool, bool)>, // (strike, is_call, is_buy)
) -> impl IntoView {
    view! {
        <div class="hf-option-chain flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-chain-header flex justify-between items-center p-3 border-b border-border-soft bg-surface-subtle">
                <span class="font-bold text-xs text-text-high">"Option Chain (F&O Straddle Matrix)"</span>
                <div class="flex items-center gap-2 font-mono text-xs">
                    <span class="text-text-muted">"Underlying Spot:"</span>
                    <span class="font-bold text-emerald-400">{move || format!("₹{:.2}", spot_price.get())}</span>
                </div>
            </div>

            <div class="overflow-x-auto flex-grow">
                <table class="w-full text-xs text-left">
                    <thead class="text-text-muted border-b border-border-soft text-[10px] uppercase font-mono bg-surface-subtle">
                        <tr>
                            <th colspan="4" class="py-1 px-2 text-center border-r border-border-soft text-emerald-400">"CALLS (CE)"</th>
                            <th class="py-1 px-2 text-center bg-surface-base font-bold text-text-high">"STRIKE"</th>
                            <th colspan="4" class="py-1 px-2 text-center border-l border-border-soft text-rose-400">"PUTS (PE)"</th>
                        </tr>
                        <tr>
                            <th class="py-1 px-2">"Delta"</th>
                            <th class="py-1 px-2">"IV"</th>
                            <th class="py-1 px-2">"OI"</th>
                            <th class="py-1 px-2 border-r border-border-soft">"LTP"</th>
                            <th class="py-1 px-2 text-center bg-surface-base">"₹"</th>
                            <th class="py-1 px-2 border-l border-border-soft">"LTP"</th>
                            <th class="py-1 px-2">"OI"</th>
                            <th class="py-1 px-2">"IV"</th>
                            <th class="py-1 px-2">"Delta"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-border-soft/40 font-mono">
                        <For
                            each=move || pairs.get()
                            key=|pair| format!("{:.2}", pair.strike)
                            children=move |pair| {
                                let strike = Signal::derive(move || pair.strike);
                                view! {
                                    <HfOptionChainRow
                                        strike
                                        spot_price=spot_price
                                        call=pair.call
                                        put=pair.put
                                        on_select_leg=on_select_leg
                                    />
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}
