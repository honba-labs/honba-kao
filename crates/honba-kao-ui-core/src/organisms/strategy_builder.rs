use crate::molecules::fno::{HfStrategyLegRow, StrategyLeg};
use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[component]
pub fn HfStrategyBuilder(
    #[prop(into)] legs: RwSignal<Vec<StrategyLeg>>,
    #[prop(into)] max_profit: Signal<Option<f64>>,
    #[prop(into)] max_loss: Signal<Option<f64>>,
    #[prop(into)] breakeven_points: Signal<Vec<f64>>,
    #[prop(into)] on_execute_strategy: Callback<()>,
) -> impl IntoView {
    let remove_leg_at = move |idx: usize| {
        legs.update(|l| {
            if idx < l.len() {
                l.remove(idx);
            }
        });
    };

    view! {
        <div class="hf-strategy-builder flex flex-col gap-3 p-4 bg-surface-base border border-border-soft rounded">
            <div class="flex justify-between items-center pb-2 border-b border-border-soft">
                <div>
                    <h3 class="text-xs font-bold text-text-high">"Multi-Leg Strategy Builder"</h3>
                    <p class="text-[11px] text-text-muted">"Construct custom spreads, straddles, iron condors"</p>
                </div>
                <div class="flex gap-3 text-xs font-mono">
                    <div>
                        <span class="text-text-muted">"Max Profit: "</span>
                        <span class="text-emerald-400 font-bold">
                            {move || match max_profit.get() {
                                Some(p) => format!("₹{:.2}", p),
                                None => "Unlimited".to_string(),
                            }}
                        </span>
                    </div>
                    <div>
                        <span class="text-text-muted">"Max Loss: "</span>
                        <span class="text-rose-400 font-bold">
                            {move || match max_loss.get() {
                                Some(l) => format!("₹{:.2}", l),
                                None => "Unlimited".to_string(),
                            }}
                        </span>
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-1.5">
                <For
                    each=move || {
                        legs.get().into_iter().enumerate().collect::<Vec<_>>()
                    }
                    key=|(idx, leg)| format!("{}-{}-{}-{}", idx, leg.side, leg.strike, leg.lots)
                    children=move |(idx, leg)| {
                        let leg_sig = Signal::derive(move || leg.clone());
                        view! {
                            <HfStrategyLegRow
                                leg=leg_sig
                                on_remove=Callback::new(move |_| remove_leg_at(idx))
                            />
                        }
                    }
                />
            </div>

            <div class="flex justify-between items-center pt-2 border-t border-border-soft text-xs">
                <div class="text-text-muted font-mono">
                    <span>"Breakeven: "</span>
                    <span>
                        {move || {
                            let bes = breakeven_points.get();
                            if bes.is_empty() {
                                "-".to_string()
                            } else {
                                bes.iter().map(|b| format!("₹{:.2}", b)).collect::<Vec<_>>().join(", ")
                            }
                        }}
                    </span>
                </div>
                <HfButton
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| on_execute_strategy.run(()))
                >
                    "Execute All Legs"
                </HfButton>
            </div>
        </div>
    }
}
