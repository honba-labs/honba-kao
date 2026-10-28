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
    #[prop(into, optional)] total_collateral_value: Option<Signal<f64>>,
    #[prop(into)] margin_used: Signal<f64>,
    #[prop(into, optional)] on_deposit: Option<Callback<String>>,
    #[prop(into, optional)] on_withdraw: Option<Callback<String>>,
) -> impl IntoView {
    // Haircut evaluation: collateral value = balance × price × (1 − haircut%).
    // This is the previously-unimplemented haircut evaluation (issue #36).
    let rows = Signal::derive(move || {
        assets.get().into_iter().map(|a| {
            let haircut = a.haircut_pct.clamp(0.0, 100.0);
            let collateral_value = a.balance * a.price * (1.0 - haircut / 100.0);
            CollateralAssetRowView { asset: a, collateral_value }
        }).collect::<Vec<_>>()
    });
    let total_collateral = total_collateral_value.unwrap_or_else(|| {
        Signal::derive(move || rows.get().iter().map(|r| r.collateral_value).sum::<f64>())
    });

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
                        <span class="font-bold text-text-high">{move || format!("₹{:.2}", total_collateral.get())}</span>
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
                            each=move || rows.get()
                            key=|r| r.asset.symbol.clone()
                            children=move |r| {
                                let sym = r.asset.symbol.clone();
                                let sym_c1 = sym.clone();
                                let sym_c2 = sym;
                                view! {
                                    <tr class="hover:bg-surface-elevated">
                                        <td class="py-2 px-3 font-sans font-semibold text-text-high">{r.asset.name} <span class="text-text-muted text-[10px]">({r.asset.symbol})</span></td>
                                        <td class="py-2 px-3 text-right">{format!("{:.4}", r.asset.balance)}</td>
                                        <td class="py-2 px-3 text-right">{format!("₹{:.2}", r.asset.price)}</td>
                                        <td class="py-2 px-3 text-right text-amber-400">{format!("{:.1}%", r.asset.haircut_pct)}</td>
                                        <td class="py-2 px-3 text-right font-bold text-emerald-400">{format!("₹{:.2}", r.collateral_value)}</td>
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

#[derive(Clone, Debug)]
struct CollateralAssetRowView {
    asset: CollateralAssetRow,
    collateral_value: f64,
}
