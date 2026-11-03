//! `HfCollateralHaircutTile` — asset-level collateral contribution with haircut
//! ratio. Ported from `secured-finance-app` atoms `CollateralInformation` /
//! `CollateralInformationTable`.

use leptos::prelude::*;
use crate::primitive::HfAssetBadge;

#[component]
pub fn HfCollateralHaircutTile(
    #[prop(into)] asset: String,
    #[prop(into)] balance: f64,
    #[prop(into)] haircut_pct: f64,
    #[prop(into)] collateral_value: f64,
) -> impl IntoView {
    let eligible = haircut_pct < 100.0;
    view! {
        <div class="hf-collateral-tile flex justify-between items-start px-3 py-2.5 rounded border border-border-soft bg-surface-base text-xs">
            <div class="hf-asset flex items-center gap-2">
                <HfAssetBadge symbol=asset />
            </div>
            <div class="hf-values text-right font-mono text-text-high">
                <div>{format!("Balance: {:.2}", balance)}</div>
                <div class=if eligible { "text-text-high" } else { "text-text-muted" }>
                    {format!("Haircut: {:.0}%", haircut_pct)}
                </div>
                <div class="text-emerald-400">{format!("Collateral Value: {:.2}", collateral_value)}</div>
            </div>
        </div>
    }
}
