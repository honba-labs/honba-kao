//! `HfWalletSummaryWidget` — full account balance widget (cash, margin consumed,
//! buying power, collateral ratio). Ported from `secured-finance-app`
//! `MyWalletWidget` / `MyWalletCard`.

use leptos::prelude::*;
use crate::primitive::HfStatusLed;

#[component]
pub fn HfWalletSummaryWidget(
    #[prop(into)] cash: Signal<f64>,
    #[prop(into)] buying_power: Signal<f64>,
    #[prop(into)] margin_consumed: Signal<f64>,
    #[prop(into)] collateral_ratio: Signal<f64>,
) -> impl IntoView {
    view! {
        <div class="hf-wallet-widget rounded-lg border border-border-soft bg-surface-base p-3 shadow-sm">
            <div class="hf-wallet-header mb-2 flex items-center justify-between">
                <span class="text-xs font-semibold text-text-high uppercase tracking-wide">"Account"</span>
                <HfStatusLed live=true label="Live" />
            </div>
            <div class="hf-wallet-metrics grid grid-cols-2 gap-2">
                <div class="hf-wallet-metric bg-surface-subtle rounded p-2 border border-border-soft">
                    <span class="text-[10px] text-text-muted">"Cash"</span>
                    <div class="font-mono text-text-high">{move || format!("{:.2}", cash.get())}</div>
                </div>
                <div class="hf-wallet-metric bg-surface-subtle rounded p-2 border border-border-soft">
                    <span class="text-[10px] text-text-muted">"Buying Power"</span>
                    <div class="font-mono text-text-high">{move || format!("{:.2}", buying_power.get())}</div>
                </div>
                <div class="hf-wallet-metric bg-surface-subtle rounded p-2 border border-border-soft">
                    <span class="text-[10px] text-text-muted">"Margin Consumed"</span>
                    <div class="font-mono text-text-high">{move || format!("{:.2}", margin_consumed.get())}</div>
                </div>
                <div class="hf-wallet-metric bg-surface-subtle rounded p-2 border border-border-soft">
                    <span class="text-[10px] text-text-muted">"Collateral Ratio"</span>
                    <div class="font-mono text-text-high">{move || format!("{:.2}", collateral_ratio.get())}</div>
                </div>
            </div>
        </div>
    }
}
