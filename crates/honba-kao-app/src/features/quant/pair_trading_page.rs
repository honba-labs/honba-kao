use honba_kao_ui_core::molecules::quant::HfCointegrationBand;
use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn PairTradingPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-8 p-4 rounded border border-border-soft bg-surface-base">
                <div class="flex justify-between items-center mb-3">
                    <span class="font-bold text-sm text-text-primary">"INFY / TCS Spread"</span>
                    <span class="text-xs font-mono text-text-muted">"Hedge Ratio: 0.82 • Half-life: 14.2 bars"</span>
                </div>
                <HfCointegrationBand z_score=Signal::derive(|| 1.45) />
            </div>
            <div class="col-span-12 laptop:col-span-4 p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
                <h3 class="text-xs font-semibold text-text-primary mb-1">"Z-Score Monitor"</h3>
                <div class="text-xs text-text-muted">"Entry bands at ±2.0, exit at mean reversion."</div>
            </div>
        </HfMosaicWorkspace>
    }
}
