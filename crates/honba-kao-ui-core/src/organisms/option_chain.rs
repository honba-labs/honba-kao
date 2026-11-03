use crate::molecules::fno::{HfOptionChainRow, OptionLegQuote};
use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq)]
pub struct OptionStrikePair {
    pub strike: Decimal,
    pub call: OptionLegQuote,
    pub put: OptionLegQuote,
}

#[component]
pub fn HfOptionChain(
    #[prop(into)] spot_price: Signal<Decimal>,
    #[prop(into)] pairs: Signal<Vec<OptionStrikePair>>,
    #[prop(into)] on_select_leg: Callback<(Decimal, bool, bool)>, // (strike, is_call, is_buy)
    #[prop(into, optional)] is_buy: Option<Signal<bool>>,
) -> impl IntoView {
    let is_buy = RwSignal::new(is_buy.as_ref().map(|s| s.get()).unwrap_or(true));

    view! {
        <div class="hf-option-chain flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-chain-header flex justify-between items-center p-3 border-b border-border-soft bg-surface-subtle">
                <span class="font-bold text-xs text-text-high">"Option Chain (F&O Straddle Matrix)"</span>
                <div class="flex items-center gap-2 font-mono text-xs">
                    <span class="text-text-muted">"Underlying Spot:"</span>
                    <span class="font-bold text-emerald-400">{move || format!("₹{}", spot_price.get().round_dp(2))}</span>
                </div>
                <div class="flex items-center gap-1 border-l border-border-soft pl-3">
                    <span class="text-[10px] text-text-muted">"Side:"</span>
                    <HfButton
                        class=move || if is_buy.get() { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Xxs
                        on_click=Callback::new(move |_| is_buy.set(true))
                    >
                        "Buy"
                    </HfButton>
                    <HfButton
                        class=move || if !is_buy.get() { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Xxs
                        on_click=Callback::new(move |_| is_buy.set(false))
                    >
                        "Sell"
                    </HfButton>
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
                            key=|pair| format!("{}", pair.strike.round_dp(2))
                            children=move |pair| {
                                let spot = spot_price.get();
                                view! {
                                    <HfOptionChainRow
                                        strike=pair.strike
                                        spot_price=spot
                                        call=pair.call
                                        put=pair.put
                                        on_select_leg=on_select_leg
                                        is_buy=is_buy
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
