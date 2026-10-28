use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CollateralAssetRow {
    pub symbol: String,
    pub name: String,
    pub balance: f64,
    pub price: f64,
    pub haircut_pct: f64,
    pub collateral_value: f64,
}

#[component]
pub fn HfCollateralOrganism(
    #[prop(into)] assets: Signal<Vec<CollateralAssetRow>>,
    #[prop(into)] total_collateral_value: Signal<f64>,
    #[prop(into)] margin_used: Signal<f64>,
    #[prop(into, optional)] on_deposit: Option<Callback<String>>,
    #[prop(into, optional)] on_withdraw: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="hf-collateral-organism flex flex-col gap-3 p-4 bg-surface-base border border-border-soft rounded">
            <div class="flex justify-between items-center pb-3 border-b border-border-soft">
                <div>
                    <h2 class="text-sm font-bold text-text-high">"Collateral & Margin Management"</h2>
                    <p class="text-xs text-text-muted">"Institutional multi-asset haircut and borrowing capacity"</p>
                </div>
                <div class="flex gap-4 text-xs font-mono">
                    <div>
                        <span class="text-text-muted">"Total Collateral: "</span>
                        <span class="font-bold text-text-high">{move || format!("₹{:.2}", total_collateral_value.get())}</span>
                    </div>
                    <div>
                        <span class="text-text-muted">"Margin In Use: "</span>
                        <span class="font-bold text-amber-400">{move || format!("₹{:.2}", margin_used.get())}</span>
                    </div>
                </div>
            </div>

            <div class="overflow-x-auto">
                <table class="w-full text-xs text-left">
                    <thead class="text-text-muted border-b border-border-soft text-[11px] uppercase">
                        <tr>
                            <th class="py-2 px-3">"Asset"</th>
                            <th class="py-2 px-3 text-right">"Balance"</th>
                            <th class="py-2 px-3 text-right">"Price"</th>
                            <th class="py-2 px-3 text-right">"Haircut"</th>
                            <th class="py-2 px-3 text-right">"Collateral Value"</th>
                            <th class="py-2 px-3 text-center">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-border-soft/40 font-mono">
                        <For
                            each=move || assets.get()
                            key=|a| a.symbol.clone()
                            children=move |a| {
                                let sym = a.symbol.clone();
                                let sym_c1 = sym.clone();
                                let sym_c2 = sym;
                                view! {
                                    <tr class="hover:bg-surface-elevated">
                                        <td class="py-2 px-3 font-sans font-semibold text-text-high">{a.name} <span class="text-text-muted text-[10px]">({a.symbol})</span></td>
                                        <td class="py-2 px-3 text-right">{format!("{:.4}", a.balance)}</td>
                                        <td class="py-2 px-3 text-right">{format!("₹{:.2}", a.price)}</td>
                                        <td class="py-2 px-3 text-right text-amber-400">{format!("{:.1}%", a.haircut_pct)}</td>
                                        <td class="py-2 px-3 text-right font-bold text-emerald-400">{format!("₹{:.2}", a.collateral_value)}</td>
                                        <td class="py-2 px-3 text-center">
                                            <div class="flex items-center justify-center gap-1">
                                                {on_deposit.map(|cb| view! {
                                                    <HfButton
                                                        variant=ButtonVariant::Tertiary
                                                        size=ButtonSize::Xxs
                                                        on_click=Callback::new(move |_| cb.run(sym_c1.clone()))
                                                    >
                                                        "Deposit"
                                                    </HfButton>
                                                })}
                                                {on_withdraw.map(|cb| view! {
                                                    <HfButton
                                                        variant=ButtonVariant::Ghost
                                                        size=ButtonSize::Xxs
                                                        on_click=Callback::new(move |_| cb.run(sym_c2.clone()))
                                                    >
                                                        "Withdraw"
                                                    </HfButton>
                                                })}
                                            </div>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}
