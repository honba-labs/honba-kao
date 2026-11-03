//! `HfCollateralAssetPane` — left panel listing deposited collateral asset
//! positions. Ported from `secured-finance-app` molecule `CollateralTabLeftPane`.

use leptos::prelude::*;
use crate::primitive::HfAssetBadge;

/// A single deposited collateral asset row.
#[derive(Clone, Debug, PartialEq)]
pub struct CollateralAsset {
    pub symbol: String,
    pub balance: f64,
    pub haircut_pct: f64,
    pub collateral_value: f64,
    pub eligible: bool,
}

/// Left panel of the collateral manager listing deposited asset positions.
#[component]
pub fn HfCollateralAssetPane(
    #[prop(into)] assets: Signal<Vec<CollateralAsset>>,
) -> impl IntoView {
    let total = Signal::derive(move || {
        assets.get().iter().map(|a| a.collateral_value).sum::<f64>()
    });

    view! {
        <div class="hf-collateral-asset-pane flex flex-col gap-2 p-3 rounded border border-border-soft bg-surface-base text-xs">
            <div class="hf-pane-header flex justify-between items-center mb-1 border-b border-border-soft pb-1">
                <span class="font-semibold text-text-high uppercase tracking-wider text-[10px]">"Deposited Assets"</span>
                <span class="font-mono text-text-high">{move || format!("Total: ₹{:.2}", total.get())}</span>
            </div>
            <div class="hf-asset-list flex flex-col gap-1 overflow-y-auto max-h-[300px]">
                {move || {
                    assets.get().iter().map(|asset| {
                        let eligible = asset.eligible;
                        view! {
                            <div class="hf-asset-row flex justify-between items-center px-2 py-1.5 rounded border border-border-soft/50 bg-surface-subtle/50">
                                <div class="hf-asset-info flex items-center gap-2">
                                    <HfAssetBadge symbol=asset.symbol.clone() />
                                </div>
                                <div class="hf-asset-values text-right font-mono text-text-high">
                                    <div>{format!("{:.2}", asset.balance)}</div>
                                    <div class=if eligible { "text-text-high" } else { "text-text-muted" }>
                                        {format!("haircut {:.0}%", asset.haircut_pct)}
                                    </div>
                                    <div class="text-emerald-400">{format!("₹{:.2}", asset.collateral_value)}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}
