//! `HfLiquidationProgressBar` — tri-tier collateral safety meter with liquidation
//! buffer indicator. Ported from the restructure spec (Section 7.3) and
//! `secured-finance-app` molecule `LiquidationProgressBar`.

use leptos::prelude::*;

#[component]
pub fn HfLiquidationProgressBar(
    #[prop(into)] current_utilization: Signal<f64>, // 0.0 to 100.0%
    #[prop(into)] liquidation_threshold: Signal<f64>, // e.g. 85.0%
    #[prop(into)] est_liquidation_price: Signal<Option<f64>>,
) -> impl IntoView {
    let clamped_utilization = Signal::derive(move || current_utilization.get().clamp(0.0, 100.0));
    let color_class = Signal::derive(move || {
        let u = current_utilization.get();
        if u < 60.0 {
            "hf-risk-safe text-emerald-400"
        } else if u < 80.0 {
            "hf-risk-warning text-amber-400"
        } else {
            "hf-risk-critical text-rose-500"
        }
    });

    view! {
        <div class="hf-liquidation-progress-bar flex flex-col gap-1 p-2 rounded bg-surface-base border border-border-soft">
            <div class="hf-risk-header flex justify-between text-xs">
                <span class="text-text-muted">"Margin Utilization / Liquidation Risk"</span>
                <span class=move || color_class.get()>
                    {move || format!("{:.1}%", current_utilization.get())}
                </span>
            </div>
            <div class="hf-progress-track relative w-full h-2 rounded bg-surface-subtle overflow-hidden">
                <div
                    class=move || format!("hf-progress-fill h-full transition-all {}", color_class.get())
                    style=move || format!("width: {}%;", clamped_utilization.get())
                />
                <div
                    class="hf-threshold-tick absolute top-0 bottom-0 w-0.5 bg-rose-500"
                    style=move || format!("left: {}%;", liquidation_threshold.get())
                    title="Liquidation Threshold"
                />
            </div>
            <div class="hf-risk-footer flex justify-between text-[11px] text-text-muted">
                <span>{move || format!("Buffer: +{:.1}%", (liquidation_threshold.get() - current_utilization.get()).max(0.0))}</span>
                <span>
                    {move || match est_liquidation_price.get() {
                        Some(px) => format!("Est. Liq Px: ${:.2}", px),
                        None => "Est. Liq Px: N/A".to_string(),
                    }}
                </span>
            </div>
        </div>
    }
}
